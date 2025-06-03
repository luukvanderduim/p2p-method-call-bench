use argh::FromArgs;
use atspi::{
    Role,
    connection::set_session_accessibility,
    proxy::accessible::{AccessibleProxy, ObjectRefExt},
    zbus::proxy::CacheProperties,
};
use futures::executor::block_on;
use futures::future::try_join_all;
use std::vec;
use zbus::{Connection, Message, names::BusName};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const REGISTRY_DEST: &str = "org.a11y.atspi.Registry";
const ACCESSIBLE_ROOT_PATH: &str = "/org/a11y/atspi/accessible/root";
const ACCESSIBLE_INTERFACE: &str = "org.a11y.atspi.Accessible";
const APPLICATION_INTERFACE: &str = "org.a11y.atspi.Application";

#[derive(Debug, PartialEq, Eq, Clone)]
struct A11yNode {
    role: Role,
    children: Vec<A11yNode>,
}

impl A11yNode {
    async fn from_accessible_proxy_iterative(ap: AccessibleProxy<'_>) -> Result<A11yNode> {
        let connection = ap.inner().connection().clone();

        // Contains the processed `A11yNode`'s.
        let mut nodes: Vec<A11yNode> = Vec::new();

        // Contains the `AccessibleProxy` yet to be processed.
        let mut stack: Vec<AccessibleProxy> = vec![ap];

        // If the stack has an `AccessibleProxy`, we take the last.
        while let Some(ap) = stack.pop() {
            let Ok(child_objects) = ap.get_children().await else {
                eprintln!(
                    "warn: {} on {} could not get children",
                    ap.inner().path(),
                    ap.inner().destination()
                );
                continue;
            };

            let mut children_proxies = try_join_all(
                child_objects
                    .into_iter()
                    .map(|child| child.into_accessible_proxy(&connection)),
            )
            .await?;

            let roles = try_join_all(children_proxies.iter().map(|child| child.get_role())).await?;
            stack.append(&mut children_proxies);

            let children = roles
                .into_iter()
                .map(|role| A11yNode {
                    role,
                    children: Vec::new(),
                })
                .collect::<Vec<_>>();

            let role = ap.get_role().await?;
            nodes.push(A11yNode { role, children });
        }

        let mut fold_stack: Vec<A11yNode> = Vec::with_capacity(nodes.len());

        while let Some(mut node) = nodes.pop() {
            if node.children.is_empty() {
                fold_stack.push(node);
                continue;
            }

            // If the node has children, we fold in the children from 'fold_stack'.
            // There may be more on 'fold_stack' than the node requires.
            let begin = fold_stack.len().saturating_sub(node.children.len());
            node.children = fold_stack.split_off(begin);
            fold_stack.push(node);
        }

        fold_stack.pop().ok_or("No root node built".into())
    }

    fn node_count(&self) -> u32 {
        let mut count = 0;
        let mut stack = Vec::new();

        stack.push(self.clone());

        while let Some(node) = stack.pop() {
            count += 1;
            stack.extend(node.children);
        }

        count
    }
}

async fn get_registry_accessible<'a>(conn: &Connection) -> Result<AccessibleProxy<'a>> {
    let registry = AccessibleProxy::builder(conn)
        .destination(REGISTRY_DEST)?
        .path(ACCESSIBLE_ROOT_PATH)?
        .interface(ACCESSIBLE_INTERFACE)?
        .cache_properties(CacheProperties::No)
        .build()
        .await?;

    Ok(registry)
}

async fn get_root_accessible<'c>(
    bus_name: BusName<'c>,
    conn: &'c Connection,
) -> Result<AccessibleProxy<'c>> {
    let root_accessible = AccessibleProxy::builder(conn)
        .destination(bus_name)?
        .path(ACCESSIBLE_ROOT_PATH)?
        .interface(ACCESSIBLE_INTERFACE)?
        .cache_properties(CacheProperties::No)
        .build()
        .await?;

    Ok(root_accessible)
}

/// Select the bus name to be used
#[derive(FromArgs)]
struct AccessibleBusName {
    /// the bus name or application name to be used
    /// (default: org.a11y.atspi.Registry)
    #[argh(positional, default = "String::new()")]
    bus_name: String,
}

/// Parse the bus name from the command line argument
fn parse_bus_name(name: String, conn: &Connection) -> Result<Vec<(String, BusName<'static>)>> {
    // If the name is empty, use the default bus name
    if name.is_empty() {
        let bus_name = match BusName::try_from(REGISTRY_DEST) {
            Ok(name) => name.to_owned(),
            Err(e) => return Err(format!("Invalid bus name: {REGISTRY_DEST} ({e})").into()),
        };

        return Ok(vec![(REGISTRY_DEST.to_string(), bus_name)]);
    }

    match BusName::try_from(name.clone()) {
        Ok(bus_name) => Ok(vec![(name, bus_name.to_owned())]),
        _ => {
            // If the name is not a valid bus-name, try find it as an application name
            from_app_name(name, conn)
        }
    }
}

fn get_user_yn_response(question: &str) -> Result<bool> {
    println!("{question} (Y/n)");
    let mut answer = String::new();
    std::io::stdin()
        .read_line(&mut answer)
        .expect("Failed to read line");
    let answer = answer.trim().to_lowercase();
    if answer == "y" || answer == "yes" || answer.is_empty() {
        Ok(true)
    } else if answer == "n" || answer == "no" {
        Ok(false)
    } else {
        Err(format!("Invalid response: {answer}").into())
    }
}

/// BusName from application name
fn from_app_name(
    sought_after: String,
    conn: &Connection,
) -> Result<Vec<(String, BusName<'static>)>> {
    let registry_accessible = block_on(get_registry_accessible(conn)).map_err(|e| e.to_string())?;
    let mut apps = block_on(registry_accessible.get_children()).map_err(|e| e.to_string())?;
    // get apps in reverse order - most recently entered apps first
    apps.reverse();

    // We might find multiple applications with the same name, so we want to ask the user about each
    // of them. We will store the matching applications here.
    let mut matching_apps: Vec<(String, BusName<'static>)> = Vec::new();

    for app in apps {
        let bus_name = app.name.to_owned();
        let acc_proxy = block_on(app.into_accessible_proxy(conn));
        let acc_proxy = match acc_proxy {
            Ok(acc_proxy) => acc_proxy,
            Err(e) => {
                eprintln!(
                    "warn: {} could not convert to accessible proxy: {}",
                    &bus_name, e
                );
                continue;
            }
        };

        let name = match block_on(acc_proxy.name()) {
            Ok(name) => name,
            Err(e) => {
                eprintln!("warn: {:?} returned an error getting name: {e}", &bus_name);
                continue;
            }
        };

        match (
            name == sought_after,
            name.to_lowercase() == sought_after.to_lowercase(),
            name.to_lowercase().contains(&sought_after.to_lowercase()),
        ) {
            // Perfect match
            (true, _, _) => matching_apps.push((name, bus_name.into())),

            // Case-insensitive match
            (false, true, _) => {
                println!("Sought {sought_after}, found application: {name}");

                if get_user_yn_response("Would you like to add this application?")? {
                    matching_apps.push((name, bus_name.into()));
                } else {
                    continue;
                }
            }

            // Case-insensitive partial match
            (false, false, true) => {
                println!("Sought {sought_after}, partially matches application: {name}");
                if get_user_yn_response("Would you like to add this application?")? {
                    matching_apps.push((name, bus_name.into()));
                } else {
                    continue;
                }
            }
            // No match
            (false, false, false) => {
                continue;
            }
        };
    }

    if matching_apps.is_empty() {
        return Err(format!("No application found with name: {sought_after}").into());
    }
    Ok(matching_apps)
}

#[tokio::main]
async fn main() -> Result<()> {
    set_session_accessibility(true).await?;

    let a11y = atspi::AccessibilityConnection::new().await?;
    let conn = a11y.connection();

    let args: AccessibleBusName = argh::from_env();

    // Sometimes applications have multiple connections
    // represented by multiple bus names.
    let applications = parse_bus_name(args.bus_name.clone(), conn)?;

    if applications.is_empty() {
        return Err("No application found".into());
    }

    let app = applications.first().unwrap();

    let (_name, bus_name) = app;

    let now = std::time::Instant::now();
    let acc_proxy = get_root_accessible(bus_name.clone(), conn).await?;
    let bus_tree = A11yNode::from_accessible_proxy_iterative(acc_proxy).await?;
    let bus_duration = now.elapsed();

    // Get private bus socket address
    // busctl call --address='unix:path=/run/user/1000/at-spi/bus\_0' ':1.124' '/org/a11y/atspi/accessible/root' 'org.a11y.atspi.Application' 'GetApplicationBusAddress'
    let msg: Message = conn
        .call_method(
            Some(bus_name.clone()),
            ACCESSIBLE_ROOT_PATH,
            Some(APPLICATION_INTERFACE),
            "GetApplicationBusAddress",
            &[""],
        )
        .await?;

    let socket: String = msg.body().deserialize()?;
    let socket = socket
        .strip_prefix("unix:path=")
        .map(|s| s.to_string())
        .unwrap_or(socket);

    let unix_stream = tokio::net::UnixStream::connect(socket.trim())
        .await
        .map_err(|e| format!("Error building UnixStream from socket: {e}"))?;

    let conn2: zbus::Connection = zbus::connection::Builder::unix_stream(unix_stream)
        .p2p()
        .build()
        .await
        .map_err(|e| format!("Error building connection: {e}"))?;

    let now = std::time::Instant::now();
    let acc_proxy = get_root_accessible(bus_name.clone(), &conn2).await?;
    let p2p_tree = A11yNode::from_accessible_proxy_iterative(acc_proxy).await?;
    let p2p_duration = now.elapsed();

    println!("The tree counts should be the same.");
    println!(
        "{:<70} {:>15.2?}",
        "Bus tree node count:",
        bus_tree.node_count()
    );
    println!(
        "{:<70} {:>15.2?}",
        "P2P tree node count:",
        p2p_tree.node_count()
    );
    println!();

    println!("{:<70} {:>15.2?}", "Bus connection time:", bus_duration);
    println!("{:<70} {:>15.2?}", "P2P connection time:", p2p_duration);
    println!(
        "{:<70} {:>15.2?}",
        "P2P speedup:",
        bus_duration.as_secs_f64() / p2p_duration.as_secs_f64()
    );

    Ok(())
}
