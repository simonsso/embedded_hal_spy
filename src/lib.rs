#![no_std]
extern crate embedded_hal as hal;
use core::cell::RefCell;
/// embedded hal spy implemnets call backs for used traits
///
/// Intended use is chaining over an existing embedded_hal
/// implementation sniffing all the data. Useful when preparing
/// for a refacforing and want to collect actual data for unit
/// test case.
///

/// Blocking SPI API
/// hal::blocking::spi::Transfer will return
/// DataWord::First at start of transfer, all data sent in DataWord::Byte(u8)
/// DataWord::Response to indicate where transmit ends and response begins
/// all recived bytes in DataWord::Byte(u8) ending with DataWord::Last
///
/// Usage:
/// ```
///    extern crate embedded_hal_spy;
///    use embedded_hal_spy::DataWord;
/// #   use linux_embedded_hal::Spidev;
/// #   use linux_embedded_hal::Pin;
///
/// #   if let Ok(spi) = Spidev::open("/dev/spidev0.0"){
///    let mut spix = embedded_hal_spy::new(spi,
///             |w|{
///                 match w {
///                     DataWord::First => {
///                         print!("data = [");
///                     },
///                     DataWord::Last =>  {println!("],"); },
///                     DataWord::Response =>  { print!("],\r\n       ["); },
///
///                     DataWord::Byte(num) => {
///                         print!("{:x},",num);
///                         },
///                     _other => {},
///                 }
///             }
///      );
/// #     }
/// ```
pub struct Spy<T, F>
where
    F: Fn(DataWord),
{
    /// object implementing emedded hal
    s: RefCell<T>,
    /// Callback
    f: RefCell<F>,
}
/// Chain existing embedded_hal trait implementation to
/// embedded_hal_spy
pub fn new<T, F>(s: T, f: F) -> Spy<T, F>
where
    F: Fn(DataWord),
{
    Spy {
        s: RefCell::new(s),
        f: RefCell::new(f),
    }
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
    /// hal::digital::ToggleableOutput return value
    Toggle,
}

use hal::spi::FullDuplex;
extern crate nb;
/// FullDuplex will return every data sent and read in DataWord::Byte(u8)
///
impl<T, F> FullDuplex<u8> for Spy<T, F>
where
    T: FullDuplex<u8>,
    F: Fn(DataWord),
{
    type Error = T::Error;
    fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
        let mut s = self.s.borrow_mut();
        let ans = s.read();
        match &ans {
            Ok(w) => {
                (self.f.borrow_mut())(DataWord::Byte(w.clone()));
            }
            _other => {}
        }
        ans
    }
    fn send(&mut self, w: u8) -> Result<(), nb::Error<Self::Error>> {
        (self.f.borrow_mut())(DataWord::Byte(w));
        let mut s = self.s.borrow_mut();
        s.send(w)
    }
}

impl<T, F> hal::blocking::spi::Transfer<u8> for Spy<T, F>
where
    T: hal::blocking::spi::Transfer<u8>,
    F: Fn(DataWord),
{
    type Error = T::Error;
    /// Sends `Word` to the slave. Returns the `Word` received from the slave
    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        (self.f.borrow_mut())(DataWord::First);
        for w in words.iter() {
            (self.f.borrow_mut())(DataWord::Byte(*w));
        }
        (self.f.borrow_mut())(DataWord::Response);
        let ans = (self.s.borrow_mut()).transfer(words)?;
        for w in ans.iter() {
            (self.f.borrow_mut())(DataWord::Byte(*w));
        }
        (self.f.borrow_mut())(DataWord::Last);

        Ok(ans)
    }
}

/// Blocking write
impl<T, F> hal::blocking::spi::Write<u8> for Spy<T, F>
where
    T: hal::blocking::spi::Write<u8>,
    F: Fn(DataWord),
{
    type Error = T::Error;
    /// Sends `words` to the slave, ignoring all the incoming words
    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        for w in words.iter() {
            (self.f.borrow_mut())(DataWord::Byte(*w));
        }
        (self.s.borrow_mut()).write(words)
    }
}
#[cfg(feature = "embedded_hal_digital_io_legacy_v1")]
/// Legacy traits
/// Digital InputPin
impl<T, F> hal::digital::v1::InputPin for Spy<T, F>
where
    T: hal::digital::v1::InputPin,
    F: Fn(DataWord),
{
    fn is_high(&self) -> bool {
        let state = (self.s.borrow_mut()).is_high();

        (self.f.borrow_mut())(DataWord::Byte(state as u8));
        state
    }
    fn is_low(&self) -> bool {
        let state = (self.s.borrow_mut()).is_low();
        (self.f.borrow_mut())(DataWord::Byte((!state) as u8));
        state
    }
}
#[cfg(feature = "embedded_hal_digital_io_legacy_v1")]
/// Digital OutputPin
impl<T, F> hal::digital::v1::OutputPin for Spy<T, F>
where
    T: hal::digital::v1::OutputPin,
    F: Fn(DataWord),
{
    fn set_high(&mut self) {
        (self.f.borrow_mut())(DataWord::Byte(1));
        (self.s.borrow_mut()).set_high()
    }
    fn set_low(&mut self) {
        (self.f.borrow_mut())(DataWord::Byte(0));
        (self.s.borrow_mut()).set_low()
    }
}
#[cfg(feature = "embedded_hal_digital_io_legacy_v1")]
impl<T, F> hal::digital::v1::ToggleableOutputPin for Spy<T, F>
where
    T: hal::digital::v1::ToggleableOutputPin,
    F: Fn(DataWord),
{
    fn toggle(&mut self) {
        (self.f.borrow_mut())(DataWord::Toggle);
        (self.s.borrow_mut()).toggle()
    }
}

#[cfg(feature = "embedded_hal_digital_io_legacy_v1")]
impl<T, F> hal::digital::v1::StatefulOutputPin for Spy<T, F>
where
    T: hal::digital::v1::StatefulOutputPin,
    F: Fn(DataWord),
{
    fn is_set_high(&self) -> bool {
        let state = (self.s.borrow_mut()).is_set_high();

        (self.f.borrow_mut())(DataWord::Byte(state as u8));
        state
    }
    fn is_set_low(&self) -> bool {
        let state = (self.s.borrow_mut()).is_set_low();
        (self.f.borrow_mut())(DataWord::Byte((!state) as u8));
        state
    }
}

#[cfg(not(feature = "embedded_hal_digital_io_legacy_v1"))]
// V2 traits
/// Digital InputPin
impl<T, F> hal::digital::v2::InputPin for Spy<T, F>
where
    T: hal::digital::v2::InputPin,
    F: Fn(DataWord),
{
    type Error = T::Error;
    fn is_high(&self) -> Result<bool, Self::Error> {
        let state = (self.s.borrow_mut()).is_high()?;
        (self.f.borrow_mut())(DataWord::Byte(state as u8));
        Ok(state)
    }
    fn is_low(&self) -> Result<bool, Self::Error> {
        let state = (self.s.borrow_mut()).is_low()?;
        (self.f.borrow_mut())(DataWord::Byte((!state) as u8));
        Ok(state)
    }
}
#[cfg(not(feature = "embedded_hal_digital_io_legacy_v1"))]
/// Digital OutputPin
impl<T, F> hal::digital::v2::OutputPin for Spy<T, F>
where
    T: hal::digital::v2::OutputPin,
    F: Fn(DataWord),
{
    type Error = T::Error;
    fn set_high(&mut self) -> Result<(), T::Error> {
        (self.f.borrow_mut())(DataWord::Byte(1));
        (self.s.borrow_mut()).set_high()
    }
    fn set_low(&mut self) -> Result<(), T::Error> {
        (self.f.borrow_mut())(DataWord::Byte(0));
        (self.s.borrow_mut()).set_low()
    }
}
#[cfg(not(feature = "embedded_hal_digital_io_legacy_v1"))]
impl<T, F> hal::digital::v2::ToggleableOutputPin for Spy<T, F>
where
    T: hal::digital::v2::ToggleableOutputPin,
    F: Fn(DataWord),
{
    type Error = T::Error;
    fn toggle(&mut self) -> Result<(), T::Error> {
        (self.f.borrow_mut())(DataWord::Toggle);
        (self.s.borrow_mut()).toggle()
    }
}

#[cfg(not(feature = "embedded_hal_digital_io_legacy_v1"))]
impl<T, F> hal::digital::v2::StatefulOutputPin for Spy<T, F>
where
    T: hal::digital::v2::StatefulOutputPin,
    F: Fn(DataWord),
{
    //type Error = T::Error;
    fn is_set_high(&self) -> Result<bool, T::Error> {
        let state = (self.s.borrow_mut()).is_set_high()?;

        (self.f.borrow_mut())(DataWord::Byte(state as u8));
        Ok(state)
    }
    fn is_set_low(&self) -> Result<bool, T::Error> {
        let state = (self.s.borrow_mut()).is_set_low()?;
        (self.f.borrow_mut())(DataWord::Byte((!state) as u8));
        Ok(state)
    }
}
