#[macro_use]
extern crate clap;

use std::io::stdout;

use clap::{App, AppSettings, Arg, Shell, SubCommand};

use flary::commands;
use flary::settings::global_user::get_valid_user;

use cloudflare::framework::auth::Credentials;
use cloudflare::framework::client::async_api::Client;
use cloudflare::framework::client::ClientConfig;
use cloudflare::framework::Environment;

fn build_app() -> App<'static, 'static> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about("Manage Cloudflare domains and DNS records")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("config")
                .about("Manage authentication configuration")
                .subcommands(vec![
                    SubCommand::with_name("auth")
                        .about("Authenticate with Cloudflare via OAuth (DNS read/write scopes)"),
                ]),
        )
        .subcommand(
            SubCommand::with_name("domains")
                .alias("domain")
                .about("Manage Cloudflare domains/zones")
                .subcommands(vec![
                    SubCommand::with_name("ls")
                        .alias("list")
                        .about("List all domains"),
                ]),
        )
        .subcommand(
            SubCommand::with_name("dns")
                .about("Manage DNS records for a domain")
                .subcommands(vec![
                    SubCommand::with_name("ls")
                        .alias("list")
                        .about("List DNS records for a domain")
                        .arg(
                            Arg::with_name("name")
                                .help("Domain name")
                                .required(true),
                        ),
                    SubCommand::with_name("add")
                        .about("Add a DNS record")
                        .long_about("Add a DNS record to a domain.\nSupported record types: A, AAAA, CNAME, TXT, MX, SRV, NS")
                        .args(&[
                            Arg::with_name("domain")
                                .help("Domain to add the record to")
                                .required(true),
                            Arg::with_name("name")
                                .help("Record name (e.g. www, @, mail)")
                                .required(true),
                            Arg::with_name("type")
                                .help("Record type (A, AAAA, CNAME, TXT, MX, SRV, NS)")
                                .required(true),
                            Arg::with_name("value")
                                .help("Record value (IP, hostname, or text)")
                                .required(true),
                            Arg::with_name("proxied")
                                .long("proxied")
                                .help("Proxy through Cloudflare (orange cloud)")
                                .takes_value(false),
                            Arg::with_name("ttl")
                                .long("ttl")
                                .help("TTL in seconds (1 = automatic)")
                                .takes_value(true)
                                .default_value("1"),
                            Arg::with_name("priority")
                                .long("priority")
                                .help("Priority for MX/SRV records")
                                .takes_value(true),
                        ]),
                    SubCommand::with_name("update")
                        .about("Update a DNS record")
                        .long_about("Update an existing DNS record by ID.\nUse `dns ls` to find the record ID.")
                        .args(&[
                            Arg::with_name("id")
                                .help("DNS record ID to update")
                                .required(true),
                            Arg::with_name("domain")
                                .help("Domain the record belongs to")
                                .required(true),
                            Arg::with_name("name")
                                .help("Record name (e.g. www, @, mail)")
                                .required(true),
                            Arg::with_name("type")
                                .help("Record type (A, AAAA, CNAME, TXT, MX, SRV, NS)")
                                .required(true),
                            Arg::with_name("value")
                                .help("Record value (IP, hostname, or text)")
                                .required(true),
                            Arg::with_name("proxied")
                                .long("proxied")
                                .help("Proxy through Cloudflare (orange cloud)")
                                .takes_value(false),
                            Arg::with_name("ttl")
                                .long("ttl")
                                .help("TTL in seconds (1 = automatic)")
                                .takes_value(true)
                                .default_value("1"),
                        ]),
                    SubCommand::with_name("rm")
                        .alias("remove")
                        .about("Remove a DNS record")
                        .long_about("Remove a DNS record by ID.\nUse `dns ls` to find the record ID.")
                        .args(&[
                            Arg::with_name("id")
                                .help("DNS record ID to remove")
                                .required(true),
                            Arg::with_name("domain")
                                .help("Domain the record belongs to")
                                .required(true),
                            Arg::with_name("yes")
                                .long("yes")
                                .help("Skip confirmation prompt")
                                .takes_value(false),
                        ]),
                ]),
        )
        .subcommand(
            SubCommand::with_name("completions")
                .about("Generate shell completion scripts")
                .arg(
                    Arg::with_name("shell")
                        .help("Shell type")
                        .required(true)
                        .possible_values(&["bash", "zsh", "fish", "powershell", "elvish"]),
                ),
        )
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let app = build_app();
    let matches = app.get_matches();

    match matches.subcommand() {
        ("config", Some(subs)) => match subs.subcommand_name() {
            Some("auth") => commands::config::auth::auth().await,
            _ => Ok(()),
        },
        ("completions", Some(subs)) => {
            let shell = subs.value_of("shell").unwrap();
            let shell = match shell {
                "bash" => Shell::Bash,
                "zsh" => Shell::Zsh,
                "fish" => Shell::Fish,
                "powershell" => Shell::PowerShell,
                "elvish" => Shell::Elvish,
                _ => anyhow::bail!("unsupported shell: {}", shell),
            };
            build_app().gen_completions_to(crate_name!(), shell, &mut stdout());
            Ok(())
        }
        _ => {
            let user = get_valid_user().await?;
            let client = Client::new(
                Credentials::from(user),
                ClientConfig::default(),
                Environment::Production,
            )?;

            match matches.subcommand() {
                ("domains", Some(subs)) => match subs.subcommand_name() {
                    Some("ls") => {
                        let zones = flary::spinner::with_spinner("Fetching domains", commands::domains::list::call_api(&client, None)).await?;
                        for zone in &zones {
                            println!("{}", &zone.name);
                        }
                        Ok(())
                    }
                    _ => Ok(()),
                },
                ("dns", Some(subs)) => match subs.subcommand() {
                    ("ls", Some(args)) => {
                        let name = args.value_of("name").unwrap();
                        let records = flary::spinner::with_spinner("Fetching DNS records", commands::dns::list::call_api(&client, name)).await?;
                        commands::dns::list::print_records(&records);
                        Ok(())
                    }
                    ("add", Some(args)) => {
                        let domain = args.value_of("domain").unwrap();
                        let record_name = args.value_of("name").unwrap();
                        let record_type = args.value_of("type").unwrap();
                        let value = args.value_of("value").unwrap();
                        let proxied = args.is_present("proxied");
                        let ttl: u32 = args.value_of("ttl").unwrap().parse().unwrap_or(1);
                        let priority: Option<u16> = args
                            .value_of("priority")
                            .and_then(|p| p.parse().ok());

                        let record = flary::spinner::with_spinner("Adding DNS record", commands::dns::add::call_api(
                            &client,
                            domain,
                            record_name,
                            record_type,
                            value,
                            proxied,
                            ttl,
                            priority,
                        )).await?;

                        println!(
                            "Created DNS record: {} {} {} (ID: {})",
                            record.name, record_type, value, record.id,
                        );
                        Ok(())
                    }
                    ("update", Some(args)) => {
                        let id = args.value_of("id").unwrap();
                        let domain = args.value_of("domain").unwrap();
                        let record_name = args.value_of("name").unwrap();
                        let record_type = args.value_of("type").unwrap();
                        let value = args.value_of("value").unwrap();
                        let proxied = args.is_present("proxied");
                        let ttl: u32 = args.value_of("ttl").unwrap().parse().unwrap_or(1);

                        let record = flary::spinner::with_spinner("Updating DNS record", commands::dns::update::call_api(
                            &client, id, domain, record_name, record_type, value, proxied, ttl,
                        )).await?;

                        println!(
                            "Updated DNS record: {} {} {} (ID: {})",
                            record.name, record_type, value, record.id,
                        );
                        Ok(())
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
