use std::{io::Write, net::TcpStream};

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use color_eyre::{eyre::eyre, Result};
use serde::Deserialize;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[arg(long, value_name = "IP", default_value = "192.168.0.209")]
    host: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    GenerateCompletions,
    #[command(alias("si"))]
    SelectInput {
        input: Input,
    },
    #[command(alias("sv"))]
    VideoSelect {
        input: Input,
    },
    GetPlayerId,
    #[command(alias("url"))]
    PlayUrl {
        #[arg(long)]
        pid: Option<i64>,
        url: String,
    },
    Text {
        command: String,
    },
    Heos {
        url: String,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Input {
    CblSat,
    #[value(alias("mplay"))]
    MediaPlayer,
    #[value(alias("bd"))]
    BluRay,
    Game,
    Aux1,
    Aux2,
    Phono,
    #[value(alias("tv"))]
    TvAudio,
    Tuner,
    Usb,
    Bluetooth,
    #[value(alias("iradio"))]
    InternetRadio,
    #[value(alias("heos"))]
    Net,
}
impl Input {
    fn to_protocol_name(self) -> &'static str {
        match self {
            Input::CblSat => "CBLSAT",
            Input::MediaPlayer => "MPLAY",
            Input::BluRay => "BD",
            Input::Game => "GAME",
            Input::Aux1 => "AUX1",
            Input::Aux2 => "AUX2",
            Input::Phono => "PHONO",
            Input::TvAudio => "TV",
            Input::Tuner => "TUNER",
            Input::Usb => "USB",
            Input::Bluetooth => "BT",
            Input::InternetRadio => "IRADIO",
            Input::Net => "NET",
        }
    }
}

struct Denon {
    host: String,
    text_session: Option<TcpStream>,
    heos_session: Option<TcpStream>,
}
impl Denon {
    fn with_host(host: String) -> Self {
        Self {
            host,
            text_session: None,
            heos_session: None,
        }
    }
    fn connect_text(&mut self) -> Result<&mut TcpStream> {
        if self.text_session.is_none() {
            self.text_session = Some(TcpStream::connect((self.host.clone(), 23))?);
        }
        Ok(self.text_session.as_mut().unwrap())
    }
    fn connect_heos(&mut self) -> Result<&mut TcpStream> {
        if self.heos_session.is_none() {
            self.heos_session = Some(TcpStream::connect((self.host.clone(), 1255))?);
        }
        Ok(self.heos_session.as_mut().unwrap())
    }
    fn select_input(&mut self, input: Input) -> Result<()> {
        Ok(writeln!(
            self.connect_text()?,
            "SI{}",
            input.to_protocol_name()
        )?)
    }
    fn video_select(&mut self, input: Input) -> Result<()> {
        Ok(writeln!(
            self.connect_text()?,
            "SV{}",
            input.to_protocol_name()
        )?)
    }
    fn text_command(&mut self, command: String) -> Result<()> {
        Ok(writeln!(self.connect_text()?, "{command}")?)
    }
    fn heos_command(&mut self, url: String) -> Result<()> {
        Ok(writeln!(self.connect_heos()?, "{url}")?)
    }
    fn get_players(&mut self) -> Result<Vec<heos::Player>> {
        let mut session = self.connect_heos()?;
        writeln!(&mut session, "heos://player/get_players")?;
        let mut de = serde_json::Deserializer::from_reader(session);
        let response = heos::Response::<Vec<heos::Player>>::deserialize(&mut de)?;
        if matches!(response.heos.result, heos::HeosResult::Fail) {
            return Err(eyre!("failed to get players: {:?}", response))
        }
        Ok(response.payload)
    }
    fn get_first_player_id(&mut self) -> Result<i64> {
        Ok(self
            .get_players()?
            .first()
            .ok_or_else(|| eyre!("no players were returned from heos"))?
            .pid)
    }
    fn play_url(&mut self, pid: Option<i64>, url: String) -> Result<()> {
        let pid = if let Some(pid) = pid {
            pid
        } else {
            self.get_first_player_id()?
        };
        let mut session = self.connect_heos()?;
        writeln!(&mut session, "heos://browse/play_stream?pid={pid}&url={url}")?;
        let mut de = serde_json::Deserializer::from_reader(session);
        let response = heos::Response::<()>::deserialize(&mut de)?;
        if matches!(response.heos.result, heos::HeosResult::Fail) {
            return Err(eyre!("failed to get players: {:?}", response))
        }
        Ok(response.payload)
    }
}

mod heos;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let host = cli.host;
    let mut denon = Denon::with_host(host);
    match cli.command {
        Command::GenerateCompletions => {
            clap_complete::generate(
                clap_complete::shells::Zsh,
                &mut Cli::command(),
                "denounce",
                &mut std::io::stdout(),
            );
        }
        Command::SelectInput { input } => {
            denon.select_input(input)?;
        }
        Command::VideoSelect { input } => {
            denon.video_select(input)?;
        }
        Command::GetPlayerId => {
            println!("{}", denon.get_first_player_id()?);
        }
        Command::PlayUrl { pid, url } => {
            denon.play_url(pid, url)?;
        }
        Command::Text { command } => {
            denon.text_command(command)?;
        }
        Command::Heos { url } => {
            denon.text_command(url)?;
        }
    }
    Ok(())
}
