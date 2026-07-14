#[macro_use]
extern crate clap;

use flary::commands;
use flary::settings;

use clap::{App, Arg, SubCommand};
use cloudflare::framework::auth::Credentials;
use cloudflare::framework::client::async_api::Client;
use cloudflare::framework::client::ClientConfig;
use cloudflare::framework::Environment;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let app = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .subcommand(
            SubCommand::with_name("config")
                .about("Configure flary settings")
                .subcommands(vec![
                    SubCommand::with_name("auth")
                        .about("Re-authorize the wrangler OAuth token with DNS permissions"),
                ]),
        )
        .subcommand(
            SubCommand::with_name("domains")
                .alias("domain")
                .about("Manage your domain names")
                .subcommands(vec![
                    SubCommand::with_name("ls")
                        .alias("list")
                        .about("List all domains"),
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
                        .args(&[
                            Arg::with_name("domain")
                                .help("Domain name to add the record to")
                                .required(true),
                            Arg::with_name("name")
                                .help("DNS record name (e.g. 'www', '@' for root)")
                                .required(true),
                            Arg::with_name("type")
                                .help("Type of DNS record (A, AAAA, CNAME, TXT, MX, SRV, NS)")
                                .required(true),
                            Arg::with_name("value")
                                .help("Value of the DNS record")
                                .required(true),
                            Arg::with_name("proxied")
                                .long("proxied")
                                .help("Whether the record is proxied through Cloudflare")
                                .takes_value(false),
                            Arg::with_name("ttl")
                                .long("ttl")
                                .help("Time to live in seconds (1 = automatic)")
                                .takes_value(true)
                                .default_value("1"),
                            Arg::with_name("priority")
                                .long("priority")
                                .help("Priority for MX/SRV records")
                                .takes_value(true),
                        ]),
                    SubCommand::with_name("update")
                        .about("Update an existing DNS record")
                        .args(&[
                            Arg::with_name("id")
                                .help("ID of the DNS record to update")
                                .required(true),
                            Arg::with_name("domain")
                                .help("Domain name the record belongs to")
                                .required(true),
                            Arg::with_name("name")
                                .help("DNS record name (e.g. 'www', '@' for root)")
                                .required(true),
                            Arg::with_name("type")
                                .help("Type of DNS record (A, AAAA, CNAME, TXT, MX, SRV, NS)")
                                .required(true),
                            Arg::with_name("value")
                                .help("Value of the DNS record")
                                .required(true),
                            Arg::with_name("proxied")
                                .long("proxied")
                                .help("Whether the record is proxied through Cloudflare")
                                .takes_value(false),
                            Arg::with_name("ttl")
                                .long("ttl")
                                .help("Time to live in seconds (1 = automatic)")
                                .takes_value(true)
                                .default_value("1"),
                        ]),
                    SubCommand::with_name("rm")
                        .alias("remove")
                        .about("Remove a DNS record")
                        .args(&[
                            Arg::with_name("id")
                                .help("ID of the DNS record to remove")
                                .required(true),
                            Arg::with_name("domain")
                                .help("Domain name the record belongs to")
                                .required(true),
                            Arg::with_name("yes")
                                .long("yes")
                                .help("Skip confirmation prompt")
                                .takes_value(false),
                        ]),
                ]),
        )
        .get_matches();

    match app.subcommand() {
        ("config", Some(subs)) => match subs.subcommand_name() {
            Some("auth") => commands::config::auth::auth().await,
            _ => Ok(()),
        },
        _ => {
            let user = settings::global_user::GlobalUser::new()?;
            let client = Client::new(
                Credentials::from(user),
                ClientConfig::default(),
                Environment::Production,
            )?;

            match app.subcommand() {
                ("domains", Some(subs)) => match subs.subcommand_name() {
                    Some("ls") => commands::domains::list(&client, None).await,
                    _ => Ok(()),
                },
                ("dns", Some(subs)) => match subs.subcommand() {
                    ("ls", Some(args)) => {
                        commands::dns::list(&client, args.value_of("name").unwrap()).await
                    }
                    ("add", Some(args)) => {
                        let domain = args.value_of("domain").unwrap();
                        let name = args.value_of("name").unwrap();
                        let record_type = args.value_of("type").unwrap();
                        let value = args.value_of("value").unwrap();
                        let proxied = args.is_present("proxied");
                        let ttl: u32 = args.value_of("ttl").unwrap().parse().unwrap_or(1);
                        let priority: Option<u16> = args
                            .value_of("priority")
                            .and_then(|p| p.parse().ok());

                        commands::dns::add(
                            &client,
                            domain,
                            name,
                            record_type,
                            value,
                            proxied,
                            ttl,
                            priority,
                        )
                        .await
                    }
                    ("update", Some(args)) => {
                        let id = args.value_of("id").unwrap();
                        let domain = args.value_of("domain").unwrap();
                        let name = args.value_of("name").unwrap();
                        let record_type = args.value_of("type").unwrap();
                        let value = args.value_of("value").unwrap();
                        let proxied = args.is_present("proxied");
                        let ttl: u32 = args.value_of("ttl").unwrap().parse().unwrap_or(1);

                        commands::dns::update(
                            &client, id, domain, name, record_type, value, proxied, ttl,
                        )
                        .await
                    }
                    ("rm", Some(args)) => {
                        let id = args.value_of("id").unwrap();
                        let domain = args.value_of("domain").unwrap();
                        let yes = args.is_present("yes");

                        commands::dns::rm(&client, id, domain, yes).await
                    }
                    _ => Ok(()),
                },
                _ => Ok(()),
            }
        }
    }
}
