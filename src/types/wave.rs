use super::Sample;
pub type Wave<T: num::Float> = Vec<Sample<T>>;
pub trait WriteWave<T: num::Float> {
    fn write(&self, p: &str);
}
impl WriteWave<f64> for Wave<f64> {
    fn write(&self, p: &str) {
        use byteorder::{ByteOrder, LittleEndian};
        use std::fs::File;
        use std::io::Write;
        use std::mem::size_of;
        println!("Writing to file {}", p);
        let mut f = File::create(p).expect("Can't create specified file");
        let mut b = vec![0u8; size_of::<Sample<f64>>() * self.len()];
        LittleEndian::write_f64_into(self, &mut b);
        f.write_all(&b).expect("Can't write to specified file");
    }
}
impl WriteWave<f32> for Wave<f32> {
    fn write(&self, p: &str) {
        use byteorder::{ByteOrder, LittleEndian};
        use std::fs::File;
        use std::io::Write;
        use std::mem::size_of;
        println!("Writing to file {}", p);
        let mut f = File::create(p).expect("Can't create specified file");
        let mut b = vec![0u8; size_of::<Sample<f32>>() * self.len()];
        LittleEndian::write_f32_into(&self, &mut b);
        f.write_all(&b).expect("Can't write to specified file");
    }
}
