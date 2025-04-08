use chord::{
    protocol::ChordResponse, request_initiator, Node, SUCCESSOR_LIST_LENGTH,
};
use cli::Args;
use std::{
    error::Error,
    net::TcpListener,
    process,
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex, RwLock,
    },
    thread,
    time::Duration,
};

mod chord;
mod cli;
mod gossip;

const SERVER_THREAD_POOL_SIZE: u8 = 10;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse()?;

    let local_listener = TcpListener::bind(format!("0.0.0.0:{}", args.get_self_port()))
        .map_err(|err| format!("cannot establish a TCP local listener: {}", err))?;

    let self_node = Node::new(args.get_public_addr());

    chord::verify_self_node_public_addr(self_node.get_public_addr(), &local_listener).map_err(
        |err| {
            format!(
                "the assigned public socket address does not correspond to the current node: {}",
                err
            )
        },
    )?;

    let self_node_successor_list =
        chord::initialize_self_node_successor_list(&self_node, &args)?;

    println!("node is running successfully");

    let self_node_predecessor: Arc<RwLock<Option<Node>>> = Arc::new(RwLock::new(None));
    let self_node_successor_list = Arc::new(RwLock::new(self_node_successor_list));

    print_self_node_core_components(
        self_node.clone(),
        Arc::clone(&self_node_predecessor),
        Arc::clone(&self_node_successor_list),
    );

    run_network_stabilization(
        self_node.clone(),
        Arc::clone(&self_node_predecessor),
        Arc::clone(&self_node_successor_list),
    );

    let server_task_sender = spawn_background_threads(SERVER_THREAD_POOL_SIZE)?;

    for stream in local_listener.incoming() {
        let request_stream = match stream {
            Ok(request_stream) => request_stream,
            Err(err) => {
                eprintln!("failed to handle the request: {}", err);
                continue;
            }
        };

        let request_handler = chord::request_handler::build_request_handler(
            request_stream,
            self_node.clone(),
            Arc::clone(&self_node_successor_list),
            Arc::clone(&self_node_predecessor),
        );

        server_task_sender.send(Box::new(request_handler)).unwrap();
    }

    Ok(())
}

/// Spawns `n` background threads to run tasks in parallel.
/// These threads remain alive as long as the main thread is running.
///
/// Tasks can be pushed and executed in these threads using the provided `Sender`.
fn spawn_background_threads(
    n: u8,
) -> Result<Sender<Box<dyn FnOnce() + Send + 'static>>, Box<dyn Error>> {
    if n == 0 {
        return Err(From::from("number of threads invalid"));
    }

    let (sender, receiver) = mpsc::channel::<Box<dyn FnOnce() + Send + 'static>>();
    let receiver = Arc::new(Mutex::new(receiver));

    for _ in 1..=n {
        let receiver = Arc::clone(&receiver);

        thread::spawn(move || loop {
            let task = {
                let receiver_lock = receiver.lock().unwrap();
                receiver_lock.recv().unwrap()
            };

            task();
        });
    }

    Ok(sender)
}

/// Runs network stabilization
/// in a separate thread.
fn run_network_stabilization(
    self_node: Node,
    self_node_predecessor: Arc<RwLock<Option<Node>>>,
    self_node_successor_list: Arc<RwLock<[Node; SUCCESSOR_LIST_LENGTH]>>,
) {
    thread::spawn(move || loop {
        let mut active_successor = None;
        let mut potential_successor = None;

        for successor in self_node_successor_list.read().unwrap().iter() {
            if let ChordResponse::Predecessor(node) =
                request_initiator::get_predecessor(successor.get_public_addr())
            {
                active_successor = Some(successor.clone());
                potential_successor = node;
                break;
            }
        }

        let active_successor = active_successor.unwrap_or_else(|| {
            eprintln!("network failure: all successor list entries are unreachable during network stabilization");
            process::exit(1);
        });

        let current_successor = match potential_successor {
            Some(potential_successor)
                if self_node.get_ring_position() == active_successor.get_ring_position()
                    || potential_successor.is_position_stictly_between(
                        self_node.get_ring_position(),
                        active_successor.get_ring_position(),
                    ) =>
            {
                // Checks if potential_successor is active.
                // If it is not active, the current successor
                // remains as the active_successor.
                if let ChordResponse::Active =
                    request_initiator::check_remote_node(potential_successor.get_public_addr())
                {
                    potential_successor
                } else {
                    active_successor
                }
            }
            _ => active_successor,
        };

        let remote_successor_list = if let ChordResponse::SuccessorList(successor_list) =
            request_initiator::notify_remote_node(&self_node, current_successor.get_public_addr())
        {
            successor_list
        } else {
            eprintln!(
                "network failure: the current successor is unreachable during network stabilization"
            );
            process::exit(1);
        };

        // Updates self_node successor list.
        let mut new_successor_list = Vec::new();
        new_successor_list.push(current_successor.clone());
        new_successor_list
            .extend_from_slice(&remote_successor_list[0..(SUCCESSOR_LIST_LENGTH - 1)]);
        {
            let mut self_node_successor_list_lock = self_node_successor_list.write().unwrap();
            *self_node_successor_list_lock = new_successor_list.try_into().unwrap();
        }

        // Checks if `self_node_predecessor` is active.
        // If not, sets `self_node_predecessor` to `NONE`.
        let self_node_predecessor_value = self_node_predecessor.read().unwrap().clone();

        if let Some(predecessor) = self_node_predecessor_value {
            if request_initiator::check_remote_node(predecessor.get_public_addr())
                != ChordResponse::Active
            {
                let mut self_node_predecessor_lock = self_node_predecessor.write().unwrap();
                *self_node_predecessor_lock = None;
            }
        }

        thread::sleep(Duration::from_secs(2));
    });
}

/// Periodically prints the current node `self_node`
/// and its Chord core components in a separate thread.
fn print_self_node_core_components(
    self_node: Node,
    self_node_predecessor: Arc<RwLock<Option<Node>>>,
    self_node_successor_list: Arc<RwLock<[Node; SUCCESSOR_LIST_LENGTH]>>,
) {
    thread::spawn(move || loop {
        println!("SELF-NODE: [{:?}]", self_node.get_public_addr());

        println!("-------------------------------------------------");

        println!(
            "PREDECESSOR: [{}]",
            match *self_node_predecessor.read().unwrap() {
                Some(ref node) => node.get_public_addr().to_string(),
                None => String::from("NONE"),
            }
        );

        println!("-------------------------------------------------");

        println!("SUCCESSOR LIST:");

        for (i, node) in self_node_successor_list.read().unwrap().iter().enumerate() {
            println!("\t {} => [{:?}]", i + 1, node.get_public_addr())
        }

        println!("#################################################");

        thread::sleep(Duration::from_secs(1));
    });
}
