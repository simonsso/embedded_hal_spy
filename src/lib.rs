#![no_std]
extern crate  embedded_hal as hal;

/// embedded hal spy implemnets call backs for used traits
/// 
/// Intended use is chaining over an existing embedded_hal
/// implementation sniffing all the data. Useful when preparing
/// for a refacforing and want to collect actual data for unit
/// test case.
/// 

pub struct Spy<T,F>
where F:FnMut(DataWord)
{
    /// object implementing emedded hal 
    pub s: T,
    /// Callback
    pub f: F,
}


/// Call back data is encapulated in enum DataWord
/// First and Last are provided from some transation
/// oriented traits to indicate first and last
pub enum DataWord {
    None,
    /// Encapsulate data
    Byte(u8),
    /// indicates first byte in transaction when used it 
    /// will be followd by last after the last byte
    First,
    /// When used it is sent after last byte in transaction 
    Last,
    /// Indicate beggining of response from a tranasction based class
    Response,
    /// embedded_hal call have failed and will report error
    Failed,
}

use hal::spi::FullDuplex;
extern crate nb;

impl<T,F> FullDuplex<u8> for Spy<T,F>
where T:FullDuplex<u8>,
      F: FnMut(DataWord),
{
    type Error = T::Error;
    fn read (&mut self) -> Result<u8, nb::Error<Self::Error>>{
        let ans = self.s.read();
        match &ans {
            Ok(w) => {(self.f)(DataWord::Byte(w.clone()));},
            _other => {},
        }
        ans
    }
    fn send(&mut self, w: u8) -> Result<(), nb::Error<Self::Error>>{
        (self.f)(DataWord::Byte(w));
        self.s.send(w)
    }
}
// Blocking SPI API

impl<T,F> hal::blocking::spi::Transfer<u8> for Spy<T,F>
where T: hal::blocking::spi::Transfer<u8>,
      F:FnMut(DataWord)
{
    type Error = T::Error;
    /// Sends `Word` to the slave. Returns the `Word` received from the slave
    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error>{
        (self.f)(DataWord::First);
        for w in words.iter(){
            (self.f)(DataWord::Byte(*w));
        }
        (self.f)(DataWord::Response);
        let ans = self.s.transfer(words)?;
        for w in ans.iter(){
            (self.f)(DataWord::Byte(*w));
        }
        (self.f)(DataWord::Last);

        Ok(ans)
    }
}

/// Blocking write
impl<T,F> hal::blocking::spi::Write<u8> for Spy<T,F>
where T: hal::blocking::spi::Write<u8>,
      F: FnMut(DataWord)
 {
    type Error = T::Error;
    /// Sends `words` to the slave, ignoring all the incoming words
    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error>{
        for w in words.iter(){
            (self.f)(DataWord::Byte(*w));
        }
        self.s.write(words)
    }
}