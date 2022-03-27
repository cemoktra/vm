use lc3::lc3::machine::LittleComputer3;
use termios::*;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    debug: bool,
    program: String,
}

fn init_terminal() -> Result<Termios, std::io::Error> {
    let termios = termios::Termios::from_fd(0)?;

    let mut new_termios = termios;
    new_termios.c_iflag &= IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL | IXON;
    new_termios.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode

    tcsetattr(0, TCSANOW, &new_termios)?;

    Ok(termios)
}

fn restore_terminal(termios: Termios) -> std::io::Result<()> {
    tcsetattr(0, TCSANOW, &termios)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let termios = init_terminal()?;

    let args = Args::parse();
    let file = std::fs::File::open(args.program)?;

    let mut lc3 = LittleComputer3::default();
    lc3.load_program(file)?;
    lc3.execute_program(args.debug)?;

    restore_terminal(termios)?;

    Ok(())
}
