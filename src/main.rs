use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use axum::{Router, Server, routing::get, extract::{Path, State as ReqState}, Extension, handler::Handler};
use axum_macros::debug_handler;
use clap::Parser;
use hyper::{Method, StatusCode};
use reqwest::Response;
use tokio::sync::Mutex;

#[derive(Parser, Debug)]
struct Args {
    pid: usize
}

#[derive(Clone, Copy)]
struct Process {
    pid: usize,
    addr: SocketAddr
}

#[derive(Clone)]
enum State {
    Leader,
    Candiadte,
    Follower,
}

#[derive(Clone)]
struct AppState {
    process: Process,
    state: State,
    processes: HashMap<usize, String>,
    leader: Option<Process>
}

impl AppState {
    pub fn set_state(&mut self, state: State) {
        self.state = state
    }
}

#[tokio::main]
async fn main() {
    let processes = HashMap::from([
        (0, String::from("127.0.0.1:3000")),
        (1, String::from("127.0.0.1:3001")),
        (2, String::from("127.0.0.1:3002")),
        (3, String::from("127.0.0.1:3003")),
        (4, String::from("127.0.0.1:3004")),
    ]);

    let args = Args::parse();

    let current_process_addr = processes.get(&args.pid).expect("the given pid is invalid");

    let current_process = Process {
        pid: args.pid,
        addr: current_process_addr.parse().expect("failed to parse process addr")
    };

    let mut state = AppState {
        process: current_process,
        state: State::Candiadte,
        processes,
        leader: None
    };

    broadcast_election(&mut state).await;

    let app = Arc::new(Mutex::new(state));

    let app = Router::new()
        .route("/election", get(handle_election_message))
        .route("/victory/:pid", get({
            move |path| handle_victory_message(path, Arc::clone(&app))
        }));

    Server::bind(&current_process.addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn broadcast_election(app: &mut AppState) {
    println!("PID={} started an election.", app.process.pid);

    if app.process.pid == get_higher_pid(&app.processes) {
        become_leader(app).await;

        return;
    }

    for (pid, addr) in app.processes.iter() {
        if *pid <= app.process.pid {
            continue;
        }

        let response = send_election_message(addr).await;

        // If received a response, no other messages must be sent
        if response.is_ok() {
            return;
        }
    }

    // If received no response, the current process must be the leader
    become_leader(app).await;
}

async fn send_election_message(process_addr: &str) -> Result<Response, reqwest::Error> {
    reqwest::Client::new()
        .post(format!("{}/election", process_addr))
        .send()
        .await
}

async fn become_leader(app: &mut AppState) {
    send_victory_mesasge(app.process, &app.processes).await;

    app.set_state(State::Leader);

    println!("PID={} became the leader.", app.process.pid)
}

async fn send_victory_mesasge(process: Process, processes: &HashMap<usize, String>) {
    for (pid, addr) in processes.iter() {
        if *pid == process.pid {
            continue;
        }

        let _ = reqwest::get(format!("{}/victory/{}", addr, process.pid))
            .await
            .expect("falied to send victory message");
    }
}

fn get_higher_pid(processes: &HashMap<usize, String>) -> usize {
    let (pid, _) = processes
        .iter()
        .max_by_key(|(pid, _)| *pid)
        .unwrap();

    *pid
}

#[debug_handler]
async fn handle_election_message(Extension(state): Extension<Arc<Mutex<AppState>>>) {
    let mut state = state.try_lock().unwrap();

    broadcast_election(&mut state).await;
}

async fn handle_victory_message(Path(pid): Path<usize>, state: Arc<Mutex<AppState>>) {
    let mut app = state.try_lock().unwrap();

    let leader = Process {
        pid,
        addr: state
            .try_lock()
            .unwrap()
            .processes
            .get(&pid)
            .unwrap()
            .parse()
            .unwrap()
    };

    app.leader = Some(leader);

    println!("PID={} is the leader now. Respect him!", pid)
}
