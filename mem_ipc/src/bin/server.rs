use clap::{arg, command, value_parser};
// use mem_ipc::{init_or_open_shm, };

fn main() {
    let matches = command!()
            .about("manages shadow copies of tables")
            .arg(
                arg!(
                    -n --name <SHM_NAME>
                )
                .help("name of shared memory to use for client/server IPC")
                .required(true)
                .value_parser(value_parser!(String))
            )
            .arg(
                arg!(
                    -l --logfile <FILE>
                )
                // We don't have syntax yet for optional options, so manually calling `required`
                .help("Enables a logfile for logging all transactions")
                .required(false)
                .value_parser(value_parser!(String))
            )
            .arg(
                arg!(
                    -d --debug ... "Turn debugging information on"
                )
            )
            .get_matches();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(name) = matches.get_one::<String>("name") {
        println!("Shared Memory: {name}");
    }

    if let Some(logfile) = matches.get_one::<String>("logfile") {
        println!("Logfile to : {}", logfile);
    }

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    match matches
        .get_one::<u8>("debug")
        .expect("Counts are defaulted")
    {
        0 => println!("Debug mode is off"),
        1 => println!("Debug mode is kind of on"),
        2 => println!("Debug mode is on"),
        _ => println!("Don't be crazy"),
    }
    // Continued program logic goes here...
}