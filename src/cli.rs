// Copyright © 2016 Felix Obenhuber
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use crate::utils;
use clap::{crate_authors, crate_name, crate_version, App, AppSettings, Arg, SubCommand};
use lazy_static::lazy_static;
use rogcat::record::Level;

lazy_static! {
    static ref ABOUT: String = {
        format!(
            "A 'adb logcat' wrapper and log processor. Your config directory is \"{}\".",
            utils::config_dir().display()
        )
    };
}

/// Build cli
pub fn cli() -> App<'static, 'static> {
    App::new(crate_name!())
          .setting(AppSettings::ColoredHelp)
          .version(crate_version!())
          .author(crate_authors!())
          .about(ABOUT.as_str())
          // Restart
          .arg(Arg::with_name("restart")
               .long("restart")
               .conflicts_with_all(&["dump", "input", "tail"])
               .help("Restart command on exit"))

          // Logcat specific
          .arg(Arg::with_name("buffer")
               .short("b")
               .long("buffer")
               .multiple(true)
               .takes_value(true)
               .conflicts_with_all(&["input", "COMMAND"])
               .help("Select specific logd buffers. Defaults to main, events, kernel and crash"))
          .arg(Arg::with_name("dev")
               .short("-s")
               .long("serial")
               .takes_value(true)
               .multiple(false)
               .help("Forwards the device selector to adb"))
          .arg(Arg::with_name("last")
               .short("L")
               .long("last")
               .conflicts_with_all(&["input", "fuchsia", "COMMAND"])
               .help("Dump the logs prior to the last reboot"))

          // Dump, head, tail
          .arg(Arg::with_name("dump")
               .short("d")
               .long("dump")
               .conflicts_with_all(&["input", "COMMAND", "restart"])
               .help("Dump the log and then exit (don't block)"))
          .arg(Arg::with_name("head")
               .short("H")
               .long("head")
               .takes_value(true)
               .conflicts_with_all(&["tail", "restart"])
               .help( "Read n records and exit"))
          .arg(Arg::with_name("tail")
               .long("tail")
               .takes_value(true)
               .conflicts_with_all(&["input", "COMMAND", "restart"])
               .help("Dump only the most recent <COUNT> lines (implies --dump)"))

          // Format
          .arg(Arg::with_name("format")
               .long("format")
               .takes_value(true)
               .possible_values(&["csv", "html", "human", "json", "raw"]).help("Output format. Defaults to human on stdout and raw on file output"))

          // Display options
          .arg(Arg::with_name("color")
               .long("color")
               .takes_value(true)
               .possible_values(&["auto", "always", "never"])
               .conflicts_with_all(&["highlight", "output"])
               .help("Terminal coloring option"))
          .arg(Arg::with_name("highlight")
               .short("h")
               .long("highlight")
               .takes_value(true)
               .multiple(true)
               .conflicts_with_all(&["output"])
               .help( "Highlight messages that match this pattern in RE2. The prefix '!' inverts the match"))
          .arg(Arg::with_name("no-dimm")
               .long("no-dimm")
               .conflicts_with("output")
               .help("Use white as dimm color"))
          .arg(Arg::with_name("bright_colors")
               .long("bright-colors")
               .conflicts_with("output")
               .help("Use intense colors in terminal output"))
          .arg(Arg::with_name("hide-timestamp")
               .long("hide-timestamp")
               .conflicts_with("output")
               .help("Hide timestamp in terminal output"))
          .arg(Arg::with_name("show-date")
               .long("show-date")
               .conflicts_with("output")
               .help("Show month and day in terminal output"))

          // Input, Output
          .arg(Arg::with_name("input")
               .short("i")
               .long("input")
               .takes_value(true)
               .multiple(true)
               .help( "Read from file instead of command. Use 'serial://COM0@115200,8N1 or similiar for reading a serial port"))
          .arg(Arg::with_name("output")
               .short("o")
               .long("output")
               .takes_value(true)
               .conflicts_with("color")
               .help("Write output to file"))
          .arg(Arg::with_name("overwrite")
               .long("overwrite")
               .requires("output")
               .help("Overwrite output file if present"))
          .arg(Arg::with_name("records-per-file")
               .short("n")
               .long("records-per-file")
               .takes_value(true)
               .requires("output")
               .help( "Write n records per file. Use k, M, G suffixes or a plain number"))
          .arg(Arg::with_name("filename-format")
               .long("filename-format")
               .short("a")
               .takes_value(true)
               .requires("output")
               .possible_values(&["single", "enumerate", "date"])
               .help( "Select a format for output file names. By passing 'single' the filename provided with the '-o' option is used (default).\
                         'enumerate' appends a file sequence number after the filename passed with '-o' option whenever a new file is created \
                         (see 'records-per-file' option). 'date' will prefix the output filename with the current local date when a new file is created"))

          // Profiles
          .arg(Arg::with_name("profiles-path")
               .short("P")
               .long("profiles-path")
               .takes_value(true)
               .help("Manually specify profile file (overrules ROGCAT_PROFILES)"))
          .arg(Arg::with_name("profile")
               .short("p")
               .long("profile")
               .takes_value(true)
               .help("Select profile"))


          // Filter
          .arg(Arg::with_name("level")
               .short("l")
               .long("level")
               .takes_value(true)
               .possible_values(Level::values()).help("Minimum level"))
          .arg(Arg::with_name("filter")
               .long("filter")
               .short("f")
               .takes_value(true)
               .multiple(true)
               .help("Regex filter on tag, pid, thread and message."))
          .arg(Arg::with_name("filter-case-insensitive")
               .long("Filter")
               .short("F")
               .takes_value(true)
               .multiple(true)
               .help("Same as -f/--filter but case insensitive"))
          .arg(Arg::with_name("message")
               .short("m")
               .long("message")
               .takes_value(true)
               .multiple(true)
               .help("Message filters in RE2. The prefix '!' inverts the match"))
          .arg(Arg::with_name("message-case-insensitive")
               .short("M")
               .long("Message")
               .takes_value(true)
               .multiple(true)
               .help("Same as -m/--message but case insensitive"))
          .arg(Arg::with_name("message_only")
             .long("message-only")
             .conflicts_with("output")
             .help("Only output message"))
          .arg(Arg::with_name("tag")
               .short("t")
               .long("tag")
               .takes_value(true)
               .multiple(true).help("Tag filters in RE2. The prefix '!' inverts the match"))
          .arg(Arg::with_name("tag-case-insensitive")
               .short("T")
               .long("Tag")
               .takes_value(true)
               .multiple(true)
               .help("Same as -t/--tag but case insensitive"))

          // Fuchsia
          .arg(Arg::with_name("fuchsia")
               .long("ffx")
               .takes_value(false)
               .conflicts_with_all(&["buffer", "dev", "input", "COMMAND", "restart", "tail"])
               .help("Use ffx log instead of adb logcat"))

          // Command
          .arg(Arg::with_name("COMMAND")
               .help( "Optional command to run and capture stdout and stdderr from. Pass \"-\" to d capture stdin'. If omitted, rogcat will run \"adb logcat -b all\" and restarts this commmand if 'adb' terminates",))
          // Bugreport
          .subcommand(SubCommand::with_name("bugreport")
                    .about("Capture bugreport. This is only works for Android versions < 7.")
                    .arg(Arg::with_name("zip").short("z").long("zip").help("Zip report"))
                    .arg(Arg::with_name("overwrite").long("overwrite").help("Overwrite report file if present"))
                    .arg(Arg::with_name("file").help("Output file name - defaults to <now>-bugreport")))
          // Completions
          .subcommand(SubCommand::with_name("completions")
                    .about("Generates completion scripts")
                    .arg(Arg::with_name("shell")
                         .required(true)
                         .possible_values(&["bash", "fish", "zsh"])
                         .help("The shell to generate the script for")))
          // Clear android log buffer
          .subcommand(SubCommand::with_name("clear")
                    .about("Clear logd buffers")
                         .arg(Arg::with_name("buffer")
                              .short("b")
                              .long("buffer")
                              .multiple(true)
                              .takes_value(true)
                              .help("Select specific log buffers to clear. Defaults to main, events, kernel and crash")))
          // List adb devices
          .subcommand(SubCommand::with_name("devices")
                    .about("List available devices"))
          // Log to logd
          .subcommand(SubCommand::with_name("log")
                    .about("Add log message(s) log buffer")
                    .arg(Arg::with_name("tag")
                         .short("t")
                         .long("tag")
                         .takes_value(true)
                         .help("Log tag"))
                    .arg(Arg::with_name("level")
                         .short("l")
                         .long("level")
                         .takes_value(true)
                         .possible_values(&[ "trace", "debug", "info", "warn", "error", "fatal", "assert", "T", "D", "I", "W", "E", "F", "A" ],)
                         .help("Log on level"))
                    .arg_from_usage("[MESSAGE] 'Log message. Pass \"-\" to read from stdin'."))
}
