use std::{collections::HashMap, net::SocketAddr, sync::{Arc, Mutex}};

use axum::{Router, Server, routing::get, extract::{Path, State as ReqState}, Extension};
use clap::Parser;
use hyper::{Method, StatusCode};
use reqwest::Response;

#[derive(Parser, Debug)]
struct Args {
    pid: usize
}

#[derive(Clone, Copy)]
struct Process {
    pid: usize,
    addr: SocketAddr
}

enum State {
    Leader,
    Candiadte,
    Follower,
}

struct AppState<'a> {
    process: Process,
    state: State,
    processes: HashMap<usize, &'a str>,
    leader: Option<Process>
}

impl<'a> AppState<'a> {
    pub fn set_state(&mut self, state: State) {
        self.state = state
    }
}

#[tokio::main]
async fn main() {
    let processes: HashMap<usize, &str> = HashMap::from([
        (0, "127.0.0.1:3000"),
        (1, "127.0.0.1:3001"),
        (2, "127.0.0.1:3002"),
        (3, "127.0.0.1:3003"),
        (4, "127.0.0.1:3004"),
    ]);

    let args = Args::parse();

    let current_process_addr = *processes.get(&args.pid).expect("the given pid is invalid");

    let current_process = Process {
        pid: args.pid,
        addr: current_process_addr.parse().expect("failed to parse process addr")
    };

    let mut state = AppState {
        process: current_process,
        state: State::Candiadte,
        processes
    };

    broadcast_election(&mut state).await;

    let app = Router::new()
        .route("/election", get(handle_election_message))
        .route("/victory/:pid", get(handle_victory_message))
        .with_state(Arc::new(Mutex::new(state)));

    Server::bind(&current_process.addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    // send election message to processes with pid higher than itself
    // if received no ok response
        // broadcast victory message

    // if received response is from process with pid higher
        // wait for victory message


    // receives election message from process with pid smaller
        // start election from the beginning

    // receives victory message
        // consider the sender as the leader
}

async fn broadcast_election<'a>(app: &mut AppState<'a>) {
    let process = app.process;
    let processes = &app.processes.clone();

    if process.pid == get_higher_pid(processes) {
        become_leader(app).await;

        return;
    }

    for (pid, addr) in processes.iter() {
        if *pid <= process.pid {
            continue;
        }

        let response = send_election_message(addr).await;

        // If received a response, no other messages must be sent
        if let Ok(response) = response {
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

async fn become_leader<'a>(app: &mut AppState<'a>) {
    send_victory_mesasge(app.process, &app.processes).await;

    app.set_state(State::Leader);
}

async fn send_victory_mesasge(process: Process, processes: &HashMap<usize, &str>) {
    for (pid, addr) in processes.iter() {
        if *pid == process.pid {
            continue;
        }

        let _ = reqwest::get(format!("{}/victory/{}", addr, process.pid))
            .await
            .expect("falied to send victory message");
    }
}

fn get_higher_pid(processes: &HashMap<usize, &str>) -> usize {
    let (pid, _) = processes
        .iter()
        .max_by_key(|(pid, _)| *pid)
        .unwrap();

    *pid
}

async fn handle_election_message<'a>(Extension(state): Extension<Arc<Mutex<AppState<'a>>>>) {
    let mut app = state.try_lock().unwrap();

    broadcast_election(&mut app).await
}

async fn handle_victory_message<'a>(Path(pid): Path<usize>, Extension(state): Extension<Arc<Mutex<AppState<'a>>>>) {
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
