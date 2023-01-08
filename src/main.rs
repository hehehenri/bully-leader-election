use std::{collections::HashMap, net::SocketAddr, sync::{Arc, Mutex}};

use axum::{Router, Server, routing::get, extract::{Path, State as ReqState}};
use clap::Parser;
use hyper::{Method, StatusCode};

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
    processes: HashMap<usize, &'a str>
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

    send_election_message(&mut state).await;

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

async fn send_election_message<'a>(app: &mut AppState<'a>) {
    let process = app.process;
    let processes = &app.processes.clone();

    if process.pid == get_higher_pid(processes) {
        become_leader(app).await;

        return;
    }

    let mut should_become_leader = true;

    for (pid, addr) in processes.iter() {
        if *pid <= process.pid {
            continue;
        }

        let response = reqwest::Client::new()
            .post(format!("{}/election", addr))
            .send()
            .await;

        if let Ok(response) = response {
            if response.status() == StatusCode::OK {
                should_become_leader = false;
            }
        }
    }

    if should_become_leader {
        become_leader(app).await;
    }
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

async fn handle_election_message<'a>(ReqState(state): ReqState<Arc<Mutex<AppState<'a>>>>) {
}

async fn handle_victory_message<'a>(Path(pid): Path<String>, ReqState(state): ReqState<Arc<Mutex<AppState<'a>>>>) {
    todo!()
}
