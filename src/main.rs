use argh::FromArgs;
use atspi::{
    Role,
    connection::set_session_accessibility,
    proxy::{
        accessible::{AccessibleProxy, ObjectRefExt},
        application::ApplicationProxy,
    },
    zbus::proxy::CacheProperties,
};
use futures::future::try_join_all;
use futures::{executor::block_on, future::join_all};
use std::vec;
use zbus::{Connection, Message, names::BusName};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const REGISTRY_WELL_KNOWN_NAME: &str = "org.a11y.atspi.Registry";
const ACCESSIBLE_ROOT_PATH: &str = "/org/a11y/atspi/accessible/root";
const ACCESSIBLE_INTERFACE: &str = "org.a11y.atspi.Accessible";
const APPLICATION_INTERFACE: &str = "org.a11y.atspi.Application";

#[derive(Debug, PartialEq, Eq, Clone)]
struct A11yNode {
    role: Option<Role>,
    children: Vec<A11yNode>,
}

impl A11yNode {
    async fn from_accessible_proxy(ap: AccessibleProxy<'_>) -> Result<A11yNode> {
        println!("Building A11yNode tree for {}", ap.inner().destination());
        let connection = ap.inner().connection().clone();
        // Contains the processed `A11yNode`'s.
        let mut nodes: Vec<A11yNode> = Vec::new();

        // Contains the `AccessibleProxy` yet to be processed.
        let mut stack: Vec<AccessibleProxy> = vec![ap];

        let mut a11yproxy_counter = 0;

        let mut previous_path = String::with_capacity(1024);
        // If the stack has an `AccessibleProxy`, we take the last.
        while let Some(ap) = stack.pop() {
            a11yproxy_counter += 1;
            println!("A11yproxy ({a11yproxy_counter}) for {}", ap.inner().path());

            let bus_name = ap.inner().destination();
            // println!("Getting ap.name().await");
            let name = ap.name().await;
            // println!("Received ap.name().await result");

            let node_name = {
                match name {
                    Ok(name) => format!("node: {name} on {bus_name}"),
                    Err(e) => {
                        eprintln!(
                            "Error getting name for {}: {e} -- continuing with next node.",
                            ap.inner().path()
                        );
                        format!("node: \"Unknown name\" on {bus_name}")
                    }
                }
            };

            if previous_path == ap.inner().path().as_str() {
                println!(
                    "Previous path is the same:\n Current path: {} is sibling or child of {previous_path}\n  These cannot be the same.",
                    ap.inner().path()
                );
                // number of children of this accessible proxy:
                let child_count = ap.get_children().await.map_err(|e| e.to_string())?.len();
                println!("{node_name} has {child_count} children.");

                return Err("Cycle detected".into());
            }

            // println!("GetChildren");
            let child_objects = ap.get_children().await;
            //println!("Received GetChildren result");

            let child_objects = match child_objects {
                // Ok can also be an empty vector, which is fine.
                Ok(children) => children,
                Err(e) => {
                    eprintln!(
                        "Error getting children of {node_name}: {e} -- continuing with next node."
                    );
                    continue;
                }
            };

            let child_count = child_objects.len();
            if child_count > 65536 {
                eprintln!("Error: Child count on {node_name} exceeds 65536, (has {child_count}).");
                return Err("Child count exceeds limit".into());
            }

            if child_objects.is_empty() {
                // If there are no children, we can get the role and continue.
                let role = ap.get_role().await.ok();

                // Create a node with the role and no children.
                nodes.push(A11yNode {
                    role,
                    children: Vec::new(),
                });
                continue;
            }

            let mut children_proxies = try_join_all(
                child_objects
                    .into_iter()
                    .map(|child| child.into_accessible_proxy(&connection)),
            )
            .await?;

            let roles = join_all(children_proxies.iter().map(|child| child.get_role())).await;
            stack.append(&mut children_proxies);
            // Now we have the role results of the child nodes, we can create `A11yNode`s for them.
            let children = roles
                .into_iter()
                .map(|role| A11yNode {
                    role: role.ok(),
                    children: Vec::new(),
                })
                .collect::<Vec<_>>();

            // Finaly get this node's role and create an `A11yNode` with it.
            let role = ap.get_role().await.ok();
            nodes.push(A11yNode { role, children });
            previous_path.clear();
            previous_path.push_str(ap.inner().path().as_str());
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
        let mut stack = vec![self.clone()];

        while let Some(node) = stack.pop() {
            count += 1;
            stack.extend(node.children);
        }

        count
    }
}

async fn get_registry_accessible<'a>(conn: &Connection) -> Result<AccessibleProxy<'a>> {
    let registry = AccessibleProxy::builder(conn)
        .destination(REGISTRY_WELL_KNOWN_NAME)?
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
        .cache_properties(CacheProperties::No)
        .build()
        .await?;

    Ok(root_accessible)
}

/// Select the bus name to be used
#[derive(FromArgs)]
struct AccessibleBusName {
    /// the bus name or application name to be used
    /// (default: xfce4-panel)
    #[argh(positional, default = "String::from(\"xfce4-panel\")")]
    bus_name: String,
}

/// Parse the bus name from the command line argument
fn parse_bus_name(name: String, conn: &Connection) -> Result<Vec<(String, BusName<'static>)>> {
    // If the name is empty, use the default bus name
    if name.is_empty() {
        let bus_name = match BusName::try_from(REGISTRY_WELL_KNOWN_NAME) {
            Ok(name) => name.to_owned(),
            Err(e) => {
                return Err(format!("Invalid bus name: {REGISTRY_WELL_KNOWN_NAME} ({e})").into());
            }
        };

        return Ok(vec![(REGISTRY_WELL_KNOWN_NAME.to_string(), bus_name)]);
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

    // Getting toolkit provider
    let app_proxy = ApplicationProxy::builder(conn)
        .destination(bus_name.clone())?
        .path(ACCESSIBLE_ROOT_PATH)?
        .cache_properties(CacheProperties::No)
        .build()
        .await?;

    let toolkit = app_proxy.toolkit_name().await?;
    let toolkit_version = app_proxy.version().await?;

    println!("{:<70} {:>15}", "Toolkit:", toolkit);
    println!("{:<70} {:>15}", "Toolkit version:", toolkit_version);
    println!();

    let now = std::time::Instant::now();
    let acc_proxy = get_root_accessible(bus_name.clone(), conn).await?;
    let bus_tree = A11yNode::from_accessible_proxy(acc_proxy).await?;
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

    let conn2: zbus::Connection = zbus::connection::Builder::address(socket.as_str())?
        .p2p()
        .build()
        .await?;

    let now = std::time::Instant::now();
    let acc_proxy = get_root_accessible(bus_name.clone(), &conn2).await?;
    let p2p_tree = A11yNode::from_accessible_proxy(acc_proxy).await?;
    let p2p_duration = now.elapsed();

    println!("The tree counts should be the same.");
    let bus_tree_node_count = bus_tree.node_count();
    let p2p_tree_node_count = p2p_tree.node_count();
    println!(
        "{:<70} {:>15.2?}",
        "Bus tree node count:", bus_tree_node_count
    );
    println!(
        "{:<70} {:>15.2?}",
        "P2P tree node count:", p2p_tree_node_count
    );
    println!();

    println!("{:<70} {:>15.2?}", "Bus connection time:", bus_duration);
    // Average time per node in the bus tree
    println!(
        "{:<70} {:>15.2?}",
        "Avg per node (Bus):",
        per_node(bus_duration, bus_tree_node_count)
    );
    println!();
    println!("{:<70} {:>15.2?}", "P2P connection time:", p2p_duration);
    // Average time per node in the P2P tree
    println!(
        "{:<70} {:>15.2?}",
        "Avg per node (P2P):",
        per_node(p2p_duration, p2p_tree_node_count)
    );
    println!();
    println!(
        "{:<70} {:>15.2?}",
        "P2P speedup:",
        bus_duration.as_secs_f64() / p2p_duration.as_secs_f64()
    );

    Ok(())
}

fn per_node(dur: std::time::Duration, count: u32) -> std::time::Duration {
    let mut dur = dur.as_nanos();
    dur /= count as u128;
    std::time::Duration::from_nanos(dur as u64)
}
