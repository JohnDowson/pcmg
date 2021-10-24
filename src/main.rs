//#![allow(unused_imports, dead_code)]
extern crate num_traits as num;
use std::{ops::Deref, time::Instant};
pub mod types;
pub mod waves;
use anyhow::Result;
use types::*;
pub mod consts;
mod live_output;
use consts::*;

fn freq(p: &Note<f64, f64>, waveform: extern "C" fn(f64) -> f64) -> Wave<f64>
where
    //W: Fn(f64) -> f64,
{
    match p {
        Note {
            silent: true,
            duration,
            ..
        } => return vec![0f64; (duration * SAMPLERATE) as usize],
        Note {
            silent: false,
            freq,
            duration,
        } => {
            let volume = 0.1;
            let lfo = LFO::new(0., 0.);
            (0..=(duration * SAMPLERATE) as usize)
                .map(|x| x as f64 / SAMPLERATE)
                .map(|t| {
                    let attack = 0.01;
                    let decay = 0.1;
                    let mut alpha = if t <= attack {
                        //attack
                        t / attack
                    } else if t >= (duration - decay) {
                        //release
                        (duration - t) / decay
                    } else {
                        //sustain
                        1.
                    };
                    alpha *= 1.0;
                    (waveform(lfo.apply(*freq, t)) * alpha) * volume
                })
                .collect()
        }
    }
}
struct Argument {
    full: String,
    short: String,
    help: String,
    arg_type: ArgState,
}

enum ArgState {
    /// "-arg" vs ""
    Presence(bool),
    /// -arg value
    Following(Option<String>),
}

struct ArgStruct {
    args: Vec<Argument>,
}
use libloading::{Library, Symbol};
use std::collections::HashMap;

type LibsInstruments<'a> = HashMap<String, Symbol<'a, extern "C" fn(f64) -> f64>>;
struct Instruments<'a> {
    count: usize,
    inner: HashMap<String, LibsInstruments<'a>>,
}
impl<'a> Instruments<'a> {
    fn new() -> Self {
        Self {
            count: 0,
            inner: HashMap::new(),
        }
    }
    fn add_lib(
        &mut self,
        lib_name: String,
        instruments: LibsInstruments<'a>,
    ) -> anyhow::Result<()> {
        match self.inner.insert(lib_name, instruments) {
            Some(_) => Err(anyhow::anyhow!("Lib already exists")),
            None => {
                self.count += 1;
                Ok(())
            }
        }
    }
    fn get(&self, lib: &str, instrument: &str) -> anyhow::Result<impl Fn(f64) -> f64> {
        if let Some(library) = self.inner.get(lib) {
            let func = library.get(instrument).ok_or_else(|| {
                anyhow::anyhow!(format!(
                    "Instrument {} not found in library {}",
                    instrument, lib
                ))
            })?;
            let func = unsafe { (func.deref() as *const extern "C" fn(f64) -> f64) };
            Ok(|freq| unsafe { func(freq) })
        } else {
            Err(anyhow::anyhow!(format!("Library {} not found", lib)))
        }
    }
}

/// Load all libraries under specified path
fn load_libraries(path: &str) -> anyhow::Result<Instruments> {
    let mut instruments = Instruments::new();
    let dir = std::fs::read_dir(path).unwrap();
    let libraries = dir.filter_map(|e| {
        // FIXME: I need to handle errors better, for now let's hope that the user won't delete the program while it's starting
        let e = e.unwrap();
        let ft = e.file_type().unwrap();
        ft.is_file().then(|| e.path())
    });
    for library in libraries {
        unsafe {
            match Library::new(&library) {
                Ok(lib) => {
                    let lib = Box::leak(Box::new(lib));
                    let lib_instruments = load_instruments(lib)?;
                    instruments.add_lib(library.to_string_lossy().to_string(), lib_instruments)?;
                }
                Err(e) => eprintln!("Failed to load library {:?} \n {}", library, e),
            }
        }
    }
    match instruments.count {
        0 => Err(anyhow::anyhow!("No libraries could be loaded")),
        _ => Ok(instruments),
    }
}

fn load_instruments(lib: &'static Library) -> Result<LibsInstruments> {
    let maybe_init: Result<Symbol<extern "C" fn() -> &'static [&'static str]>, libloading::Error>;
    unsafe {
        maybe_init = lib.get::<extern "C" fn() -> &'static [&'static str]>(b"init");
    }
    match maybe_init {
        Ok(init) => {
            let symbols = init();
            let mut instruments = HashMap::new();
            for &symbol in symbols {
                let func: Symbol<'static, extern "C" fn(f64) -> f64> =
                    unsafe { lib.get::<extern "C" fn(f64) -> f64>(symbol.as_bytes())? };
                instruments.insert(symbol.to_owned(), func);
            }
            Ok(instruments)
        }
        Err(e) => Err(e.into()),
    }
}

fn main() -> anyhow::Result<()> {
    let instruments = load_libraries("./instruments")?;
    // load instrument libraries
    /* PSEUDOCODE
    for instrument.dll in ./instruments
        let lib = lib::Library::new("./instruments/instrument.dll")?;
        let func: lib::Symbol<unsafe extern fn() -> u32> = lib.get(b"init\0")?;

    */
    // parse arguments
    // start UI thread (using druid)
    // start synth and output threads
    // or parse input file, feed notes to synth and write audio to ouptut file
    // pipes?
    let outp = std::env::args()
        .find(|a| a.ends_with(".bin"))
        .expect("No valid output file specified");
    let t1 = Instant::now();
    /* let wave = &[
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(12., 0.25),
        Note::new(12., 0.5),
        Note::new(12., 0.5),
        Note::new(10., 0.25),
        Note::new(10., 0.5),
        Note::new(10., 0.5),
        Note::new(5., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(12., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(12., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(12., 0.5),
        Note::new(10., 0.25),
        Note::new(10., 0.25),
        Note::new(10., 0.25),
        Note::new(10., 0.25),
        Note::new(10., 0.25),
        Note::new(10., 0.25),
        Note::new(10., 0.5),
        Note::new(5., 0.25),
        Note::new(5., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::silent(1.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(10., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(10., 0.5),
        Note::new(10., 0.5),
        Note::silent(2.),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(12., 0.25),
        Note::new(12., 0.5),
        Note::new(12., 0.5),
        Note::new(10., 0.25),
        Note::new(10., 0.25),
        Note::new(10., 0.25),
        Note::new(10., 0.25),
        Note::new(10., 0.5),
        Note::new(10., 0.5),
        Note::new(5., 0.25),
        Note::new(5., 0.25),
        Note::silent(1.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(10., 0.5),
        Note::new(10., 0.5),
        Note::silent(4.),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(10., 0.5),
        Note::new(10., 0.5),
        Note::silent(4.),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(10., 0.5),
        Note::new(10., 0.5),
        Note::silent(4.),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(7., 0.5),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.25),
        Note::new(7., 0.5),
        Note::new(10., 0.5),
        Note::new(10., 0.5),
        Note::silent(4.),
    ]; */
    let wave = &[Note::new(440., 1.), Note::new(450., 1.)];
    use std::ops::Deref;
    let instrument = *instruments.get("foo", "bar")?.deref();
    wave.iter()
        .map(|note| {
            freq(
                note,
                |hertz| {
                    use std::f64;
                    f64::sin(hertz)
                }, // instrument,
            )
        })
        .flatten()
        .collect::<Wave<f64>>()
        .write(&outp);
    println!("{:?}", t1.elapsed());
    Ok(())
}

/* fn mkfn(i: usize) -> impl Fn((f64, f64)) -> (f64, f64) {
    move |input| (input.0 * i as f64, input.1)
}
fn foo() -> f64 {
    let hz: f64 = 440.;
    let t: f64 = 0.1;
    let chains = vec![
        [mkfn(1), mkfn(2), mkfn(3), mkfn(4), mkfn(5)],
        [mkfn(1), mkfn(2), mkfn(3), mkfn(4), mkfn(5)],
        [mkfn(1), mkfn(2), mkfn(3), mkfn(4), mkfn(5)],
    ];
    chains
        .iter()
        .map(|c| c.iter().fold((hz, t), |input: (f64, f64), fun| fun(input)))
        .map(|c: (f64, f64)| c.0)
        .sum()
} */
