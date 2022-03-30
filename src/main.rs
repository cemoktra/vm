use lc3::lc3::machine::LittleComputer3;
use termios::*;

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

fn usage() {
    println!("Usage: lc3 [--debug] path/to/program");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let termios = init_terminal()?;

    let args: Vec<String> = std::env::args().collect();
    let (file, debug) = match args.len() {
        2 => (args[1].clone(), false),
        3 => {
            if args[1] == "--debug" {
                (args[2].clone(), true)
            } else {
                return {
                    usage();
                    Ok(())
                };
            }
        }
        _ => {
            return {
                usage();
                Ok(())
            };
        }
    };

    let file = std::fs::File::open(file)?;

    let mut lc3 = LittleComputer3::default();
    lc3.load_program(file)?;
    lc3.execute_program(debug)?;

    restore_terminal(termios)?;

    Ok(())
}
