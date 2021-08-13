#![allow(dead_code)]
#[macro_use]
extern crate clap;

use flary::commands;
use flary::settings;

use clap::{App, Arg, SubCommand};
use cloudflare::framework::async_api::Client;
use cloudflare::framework::auth::Credentials;
use cloudflare::framework::Environment;
use cloudflare::surf::Result;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let app = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        /*
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        */
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
                        // TODO: add more args groups
                        .args(&[
                            Arg::with_name("name")
                                .help("Name of the domain")
                                .required(true),
                            Arg::with_name("subdomain")
                                .help("Subdomain name of the specified domain")
                                .required(true),
                            Arg::with_name("record type")
                                .help("Type of record to add. One of (A | AAAA | ALIAS | CNAME | TXT)")
                                .required(true),
                            Arg::with_name("value").help("value").required(true),
                        ]),
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

    match app.subcommand() {
        ("domains", Some(subs)) => {
            let user = settings::global_user::GlobalUser::new()?;
            let client = Client::new(Credentials::from(user), Environment::Production)?;

            match subs.subcommand_name() {
                Some("ls") => commands::domains::list(&client, None).await?,
                Some("inspect") => println!("domains inspect"),
                _ => (),
            }
        }
        ("dns", Some(subs)) => {
            let user = settings::global_user::GlobalUser::new()?;
            let client = Client::new(Credentials::from(user), Environment::Production)?;

            match subs.subcommand() {
                ("ls", Some(args)) => {
                    commands::dns::list(&client, args.value_of("name").unwrap()).await?
                }
                ("add", Some(args)) => println!("dns add"),
                ("rm", Some(args)) => println!("dns rm"),
                _ => (),
            }
        }
        _ => (),
    }

    Ok(())
}
