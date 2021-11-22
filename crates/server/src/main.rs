use common::serde_json;
use common::Instruction;
use common::Response;
use std::io::BufRead;
use std::io::Write;
use std::net::UdpSocket;
fn main() -> Result<(), std::io::Error> {
    let db = sqlite::open(":memory:").unwrap();
    db.execute("CREATE TABLE users (name TEXT, number TEXT)")
        .unwrap();
    print!("Enter address where to bind socket to: ");
    let mut addr = String::new();
    std::io::stdout().flush()?;
    std::io::stdin().lock().read_line(&mut addr)?;
    let socket = UdpSocket::bind(addr.trim())?;
    println!("- Socket bound to {}", addr.trim());
    // we do not want to allocate 1KB slice on stack
    let mut buf = vec![0u8; 1024];
    loop {
        let (bytes, source_addr) = socket.recv_from(&mut buf)?;

        let ins = serde_json::from_slice::<Instruction>(&buf[..bytes])
            .expect("Failed to deserialize server instruction");
        match ins {
            Instruction::AddPhoneNumber { key, number } => {
                println!("- AddPhoneNumber: {} {}", key, number);
                let statement = "INSERT INTO users VALUES (:key, :number)";
                let mut statement = db.prepare(statement).unwrap();

                statement.bind_by_name(":key", key.as_str()).unwrap();
                statement.bind_by_name(":number", number.as_str()).unwrap();

                match statement.next() {
                    Ok(sqlite::State::Done) => {}
                    Err(e) => {
                        println!("Sqlite error: {}", e);
                        socket.send_to(
                            &serde_json::to_vec(&Response::Fail {
                                message: format!("Sqlite failure on adding user entry: {}", e),
                            })
                            .unwrap(),
                            source_addr,
                        )?;
                    }
                    _ => unreachable!("Statement should be executed"),
                }
            }
            Instruction::EditNumber { key, number } => {
                println!("- Edit number: {} {}", key, number);
                let statement = "UPDATE users SET number = :number WHERE name = :key";
                let mut statement = db.prepare(statement).unwrap();

                statement.bind_by_name(":key", key.as_str()).unwrap();
                statement.bind_by_name(":number", number.as_str()).unwrap();
                match statement.next() {
                    Ok(sqlite::State::Done) => {}
                    Err(e) => {
                        println!("Sqlite error: {}", e);
                        socket.send_to(
                            &serde_json::to_vec(&Response::Fail {
                                message: format!("Sqlite failure on adding user entry: {}", e),
                            })
                            .unwrap(),
                            source_addr,
                        )?;
                    }
                    _ => unreachable!("Statement should be executed"),
                }
            }
            Instruction::DeleteUser { key } => {
                println!("- Delete user {}", key);
                let statement = "DELETE FROM users WHERE name = :name";
                let mut statement = db.prepare(statement).unwrap();
                statement.bind_by_name(":name", key.as_str()).unwrap();
                match statement.next() {
                    Ok(sqlite::State::Done) => {}
                    Err(e) => {
                        println!("Sqlite error: {}", e);
                        socket.send_to(
                            &serde_json::to_vec(&Response::Fail {
                                message: format!("Failed to receive user '{}' number: {}", key, e),
                            })
                            .unwrap(),
                            source_addr,
                        )?;
                    }
                    _ => unreachable!("Statement should be executed"),
                }
            }
            Instruction::GetAllUsers => {
                let statement = "SELECT * FROM users";
                let mut vec = vec![];
                println!("- Fetching users...");
                db.iterate(statement, |pairs| {
                    let name = pairs[0].1.unwrap();
                    let number = pairs[1].1.unwrap();
                    println!("{} {}", name, number);
                    vec.push((name.to_string(), number.to_string()));
                    true
                })
                .unwrap();

                let bytes = serde_json::to_vec(&Response::AllUsers(vec)).unwrap();

                socket.send_to(&bytes, source_addr).unwrap();
            }
        }
    }
}
