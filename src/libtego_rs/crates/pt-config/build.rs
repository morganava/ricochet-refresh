// std
use std::collections::BTreeSet;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

// extern crates
use anyhow::{bail, Context, Error};
use regex::Regex;

fn main() -> Result<(), Error> {
    let pt_config_json = std::env::var("PT_CONFIG_JSON")?;
    let pt_config_json_path = PathBuf::from_str(pt_config_json.as_str())?;

    use serde_json::Value;
    let pt_config_json = std::fs::read_to_string(pt_config_json_path)?;
    let pt_config_json = pt_config_json.replace("${pt_path}", "");
    let pt_config_json = Value::from_str(pt_config_json.as_str())?;

    if let Value::Object(mut key_values) = pt_config_json {
        let mut pt_config_rs_src: Vec<u8> = Default::default();

        writeln!(
            pt_config_rs_src,
            "// Warning! This file is auto-generated from pt_config.json in the tor-expert-bundle"
        )?;
        writeln!(pt_config_rs_src, "// Modifications will not persist!")?;
        writeln!(pt_config_rs_src)?;
        writeln!(pt_config_rs_src, "// std")?;
        writeln!(pt_config_rs_src)?;
        writeln!(pt_config_rs_src, "// pluggable transports")?;
        writeln!(pt_config_rs_src, "pub struct PluggableTransport {{")?;
        writeln!(pt_config_rs_src, "    pub binary_name: &'static str,")?;
        writeln!(
            pt_config_rs_src,
            "    pub transports: &'static [&'static str],"
        )?;
        writeln!(pt_config_rs_src, "    pub options: &'static [&'static str]")?;
        writeln!(pt_config_rs_src, "}}")?;

        let mut supported_transports: BTreeSet<String> = Default::default();
        match key_values
            .remove("pluggableTransports")
            .context("missing \"pluggableTransports\" object")?
        {
            Value::Object(key_values) => {
                let pt_count = key_values.len();
                writeln!(
                    pt_config_rs_src,
                    "pub const PLUGGABLE_TRANSPORTS: [PluggableTransport; {pt_count}] = ["
                )?;

                let client_transport_plugin_pattern = Regex::new(
                    r"(?m)^ClientTransportPlugin (?<transports>[a-zA-Z_][a-zA-Z0-9_]*(,[a-zA-Z_][a-zA-Z0-9_]*)*) exec (?<binary>[^ ]*)(?<options>( [^ ^\n]+)*)$",
                )?;

                for (key, value) in key_values {
                    let value = value
                        .as_str()
                        .context("\"pluggableTransports\" members must be type String")?;

                    let caps = client_transport_plugin_pattern
                        .captures(value)
                        .context(format!("\"pluggableTransports.{key}\" must be type String"))?;
                    let transports = caps.name("transports").unwrap().as_str();
                    let transports: Vec<&str> = transports.split(',').collect();

                    for transport in &transports {
                        if !supported_transports.insert(transport.to_string()) {
                            bail!("multiple entries in \"pluggableTransports\" claim to support \"{transport}\" transport");
                        }
                    }

                    let transports: Vec<String> = transports
                        .iter()
                        .map(|transport| format!("\"{transport}\""))
                        .collect();
                    let transports = transports.join(", ");

                    let binary = caps.name("binary").unwrap().as_str();

                    let options = caps.name("options").unwrap().as_str().trim();
                    let options: String = if options.is_empty() {
                        Default::default()
                    } else {
                        let options: Vec<String> = options
                            .split(' ')
                            .map(|option| format!("\"{option}\""))
                            .collect();
                        options.join(", ")
                    };

                    writeln!(pt_config_rs_src, "    PluggableTransport {{ binary_name: \"{binary}\", transports: &[{transports}], options: &[{options}] }},")?;
                }
                writeln!(pt_config_rs_src, "];")?;
            }
            value => bail!("unexpected value for \"pluggableTransports\": {value}"),
        };

        writeln!(pt_config_rs_src)?;
        writeln!(pt_config_rs_src, "// supported transports")?;
        let supported_transports: Vec<String> = supported_transports
            .iter()
            .map(|transport| format!("\"{transport}\""))
            .collect();
        let supported_transport_count = supported_transports.len();
        let supported_transports = supported_transports.join(",");
        writeln!(pt_config_rs_src, "pub const SUPPORTED_TRANSPORTS: [&str; {supported_transport_count}] = [{supported_transports}];")?;

        writeln!(pt_config_rs_src)?;
        writeln!(pt_config_rs_src, "// bridge lines")?;
        match key_values
            .remove("bridges")
            .context("missing \"bridges\" object")?
        {
            Value::Object(key_values) => {
                for (key, value) in key_values {
                    let bridge_type = key.as_str();

                    let bridge_lines = value.as_array().unwrap();
                    let bridge_lines: Vec<String> = bridge_lines
                        .iter()
                        .map(|value| value.as_str().unwrap().to_string())
                        .collect();
                    let bridge_lines_count = bridge_lines.len();

                    for bridge_line in &bridge_lines {
                        let transport = bridge_line.split(' ').next().unwrap();
                        if !supported_transports.contains(transport) {
                            bail!("bridge line \"{bridge_line}\" requires unsupported transport");
                        }
                    }

                    let bridge_type_upper = bridge_type.to_uppercase();

                    writeln!(pt_config_rs_src, "pub const BUILTIN_{bridge_type_upper}_BRIDGE_LINES: [&str; {bridge_lines_count}] = [")?;

                    let bridge_lines: Vec<String> = bridge_lines
                        .iter()
                        .map(|value| format!("\"{value}\""))
                        .collect();
                    for bridge_line in bridge_lines {
                        writeln!(pt_config_rs_src, "        {bridge_line},")?;
                    }
                    writeln!(pt_config_rs_src, "    ];")?;
                }
            }
            value => bail!("unexpected value for \"bridges\": {value}"),
        };
        std::fs::write("src/pt_config.rs", pt_config_rs_src)?;
    } else {
        unreachable!();
    }

    Ok(())
}
