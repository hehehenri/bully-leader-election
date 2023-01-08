use std::{collections::HashMap, env, net::SocketAddr, sync::Arc};

use axum::{Router, Server, routing::{MethodRouter, get}, extract::{Path, State}};
use clap::{Arg, Parser};

#[derive(Parser, Debug)]
struct Args {
    pid: usize
}

#[derive(Clone, Copy)]
struct Process {
    pid: usize,
    addr: SocketAddr
}

struct AppState {
    process: Process
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

    let state = AppState {
        process: current_process
    };

    let app = Router::new()
        .route("/election", get(handle_election_message))
        .route("/victory/:pid", get(handle_victory_message))
        .with_state(Arc::new(state));

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

async fn handle_election_message(State(state): State<Arc<AppState>>) {
    todo!()
}

async fn handle_victory_message(Path(pid): Path<String>, State(state): State<Arc<AppState>>) {
    todo!()
}
