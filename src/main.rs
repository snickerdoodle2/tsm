use tsm::list_sessions;

fn main() {
    let sessions = list_sessions().unwrap();
    dbg!(sessions);
}
