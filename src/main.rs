#[macro_use]
extern crate clap;

use clap::{App, Arg, SubCommand};
use cloudflare::framework::HttpApiClient;

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("domains")
                .alias("domain")
                .about("Manage your domain names")
                .subcommands(vec![
                    SubCommand::with_name("ls")
                        .alias("list")
                        .about("List all domains"),
                    SubCommand::with_name("inspect")
                        .about("Show information related to a domain")
                        .arg(
                            Arg::with_name("name")
                                .help("Name of the domain")
                                .required(true),
                        ),
                ]),
        )
        .subcommand(
            SubCommand::with_name("dns")
                .about("Manage your DNS records")
                .subcommands(vec![
                    SubCommand::with_name("ls")
                        .alias("list")
                        .about("List all DNS records of a domain")
                        .arg(
                            Arg::with_name("name")
                                .help("Name of the domain")
                                .required(true),
                        ),
                    SubCommand::with_name("add")
                        .about("Add a DNS record to a domain")
                        // TODO: add more args
                        .arg(
                            Arg::with_name("name")
                                .help("Name of the domain")
                                .required(true),
                        ),
                    SubCommand::with_name("rm")
                        .about("Remove a DNS record")
                        .arg(
                            Arg::with_name("id")
                                .help("id of the DNS record to remove")
                                .required(true),
                        ),
                ]),
        )
        .get_matches();

    let config = matches.value_of("config").unwrap_or("default.conf");
    println!("Value for config: {}", config);

    //HttpApiClient::new();
}
