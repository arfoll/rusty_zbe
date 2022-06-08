extern crate clap;

use clap::{Arg, Command};

fn main() {
    // basic app information
    let app = Command::new("hello-clap")
        .version("1.0")
        .about("Says hello")
        .author("Michael Snoyman");

    // Define the name command line option
    let name_option = Arg::new("name")
        .long("name") // allow --name
        .takes_value(true)
        .help("Who to say hello to")
        .required(true);

    // now add in the argument we want to parse
    let app = app.arg(name_option);

    // extract the matches
    let matches = app.get_matches();

    // Extract the actual name
    let name = matches.value_of("name")
        .expect("This can't be None, we said it was required");

    println!("Hello, {}!", name);
}
