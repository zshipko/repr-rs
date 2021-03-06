use std::collections::BTreeMap;

mod array;
mod bytes;
mod string;

pub type Int = isize;

pub type Pair<T, U> = (T, U);
pub type Triple<T, U, V> = (T, U, V);

pub use array::Array;
pub use bytes::Bytes;
pub use string::Str;

pub use repr_derive::IrminType as Type;

pub trait Type: Sized {
    fn encode_bin<W: std::io::Write>(&self, dest: &mut W) -> std::io::Result<usize>;

    fn decode_bin<R: std::io::Read>(src: &mut R) -> std::io::Result<Self>;
}

fn encode_int<W: std::io::Write>(mut n: i64, dest: &mut W) -> std::io::Result<usize> {
    let mut count = 0;

    loop {
        if n >= 0 && n < 128 {
            count += (n as u8).encode_bin(dest)?;
            break;
        } else {
            let out = 128 | (n & 127);
            count += (out as u8).encode_bin(dest)?;
            n = n >> 7;
        }
    }

    Ok(count)

    /*if n >= 0 && n < 128 then k chars.(n)
    else
      let out = 128 lor (n land 127) in
      k chars.(out);
      aux (n lsr 7) k */
}

fn decode_int<R: std::io::Read>(src: &mut R) -> std::io::Result<Int> {
    let mut n = 0;
    let mut p = 0;
    loop {
        let i = u8::decode_bin(src)? as i64;
        n = n + ((i & 127) << p);
        if i >= 0 && i < 128 {
            return Ok(n as Int);
        } else {
            p += 7;
        }
    }

    /*let int buf ofs =
    let rec aux buf n p ofs =
      let ofs, i = int8 buf ofs in
      let n = n + ((i land 127) lsl p) in
      if i >= 0 && i < 128 then (ofs, n) else aux buf n (p + 7) ofs
    in
    aux buf 0 0 ofs */
}

impl Type for bool {
    fn encode_bin<W: std::io::Write>(&self, dest: &mut W) -> std::io::Result<usize> {
        (if *self { 255 } else { 0 }).encode_bin(dest)
    }

    fn decode_bin<R: std::io::Read>(src: &mut R) -> std::io::Result<Self> {
        let x = u8::decode_bin(src)?;
        Ok(if x == 0 { false } else { true })
    }
}

impl Type for isize {
    fn encode_bin<W: std::io::Write>(&self, dest: &mut W) -> std::io::Result<usize> {
        encode_int(*self as i64, dest)
    }

    fn decode_bin<R: std::io::Read>(src: &mut R) -> std::io::Result<Self> {
        decode_int(src).map(|x| x as isize)
    }
}

impl Type for usize {
    fn encode_bin<W: std::io::Write>(&self, dest: &mut W) -> std::io::Result<usize> {
        encode_int(*self as i64, dest)
    }

    fn decode_bin<R: std::io::Read>(src: &mut R) -> std::io::Result<Self> {
        decode_int(src).map(|x| x as usize)
    }
}

impl Type for i32 {
    fn encode_bin<W: std::io::Write>(&self, dest: &mut W) -> std::io::Result<usize> {
        (*self as u32).encode_bin(dest)
    }

    fn decode_bin<R: std::io::Read>(src: &mut R) -> std::io::Result<Self> {
        let n = u32::decode_bin(src)?;
        Ok(n as i32)
    }
}

impl Type for i64 {
    fn encode_bin<W: std::io::Write>(&self, dest: &mut W) -> std::io::Result<usize> {
        (*self as u64).encode_bin(dest)
    }

    fn decode_bin<R: std::io::Read>(src: &mut R) -> std::io::Result<Self> {
        let n = u64::decode_bin(src)?;
        Ok(n as i64)
    }
}

impl Type for () {
    fn encode_bin<W: std::io::Write>(&self, _dest: &mut W) -> std::io::Result<usize> {
        Ok(0)
    }

    fn decode_bin<R: std::io::Read>(_src: &mut R) -> std::io::Result<Self> {
        Ok(())
    }
}

impl Type for u8 {
    fn encode_bin<W: std::io::Write>(&self, dest: &mut W) -> std::io::Result<usize> {
        dest.write_all(&[*self])?;
        Ok(1)
    }

    fn decode_bin<R: std::io::Read>(src: &mut R) -> std::io::Result<Self> {
        let mut dest = [0u8; 1];
        src.read_exact(&mut dest)?;
        Ok(dest[0])
    }
}

impl Type for u16 {
    fn encode_bin<W: std::io::Write>(&self, dest: &mut W) -> std::io::Result<usize> {
        let buf = self.to_be_bytes();
        dest.write_all(&buf)?;
        Ok(2)
    }

    fn decode_bin<R: std::io::Read>(src: &mut R) -> std::io::Result<Self> {
        let mut dest = [0u8; 2];
        src.read_exact(&mut dest)?;
        Ok(u16::from_be_bytes(dest))
    }
}

impl Type for u32 {
    fn encode_bin<W: std::io::Write>(&self, dest: &mut W) -> std::io::Result<usize> {
        let buf = self.to_be_bytes();
        dest.write_all(&buf)?;
        Ok(4)
    }

    fn decode_bin<R: std::io::Read>(src: &mut R) -> std::io::Result<Self> {
        let mut dest = [0u8; 4];
        src.read_exact(&mut dest)?;
        Ok(u32::from_be_bytes(dest))
    }
}

impl Type for u64 {
    fn encode_bin<W: std::io::Write>(&self, dest: &mut W) -> std::io::Result<usize> {
        let buf = self.to_be_bytes();
        dest.write_all(&buf)?;
        Ok(8)
    }

    fn decode_bin<R: std::io::Read>(src: &mut R) -> std::io::Result<Self> {
        let mut dest = [0u8; 8];
        src.read_exact(&mut dest)?;
        Ok(u64::from_be_bytes(dest))
    }
}

impl Type for f64 {
    fn encode_bin<W: std::io::Write>(&self, dest: &mut W) -> std::io::Result<usize> {
        self.to_bits().encode_bin(dest)
    }

    fn decode_bin<R: std::io::Read>(src: &mut R) -> std::io::Result<Self> {
        let i = u64::decode_bin(src)?;
        Ok(f64::from_bits(i))
    }
}

impl Type for String {
    fn encode_bin<W: std::io::Write>(&self, dest: &mut W) -> std::io::Result<usize> {
        Str::Ref(self.as_ref()).encode_bin(dest)
    }

    fn decode_bin<R: std::io::Read>(src: &mut R) -> std::io::Result<String> {
        let i = decode_int(src)?;
        let mut x = vec![0u8; i as usize];
        src.read_exact(&mut x)?;
        match String::from_utf8(x) {
            Ok(x) => Ok(x),
            Err(_) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid string",
            )),
        }
    }
}

impl<T: Type> Type for Vec<T> {
    fn encode_bin<W: std::io::Write>(&self, dest: &mut W) -> std::io::Result<usize> {
        Array::Ref(self.as_ref()).encode_bin(dest)
    }

    fn decode_bin<R: std::io::Read>(src: &mut R) -> std::io::Result<Vec<T>> {
        let i = decode_int(src)?;
        let mut dest = Vec::new();

        for _ in 0..i as usize {
            dest.push(T::decode_bin(src)?)
        }

        Ok(dest)
    }
}

impl<K: Ord + Type, V: Type> Type for BTreeMap<K, V> {
    fn encode_bin<W: std::io::Write>(&self, dest: &mut W) -> std::io::Result<usize> {
        let i = self.len();
        i.encode_bin(dest)?;
        let mut n = 0;
        for (k, v) in self.iter() {
            n += k.encode_bin(dest)?;
            n += v.encode_bin(dest)?;
        }
        Ok(n)
    }

    fn decode_bin<R: std::io::Read>(src: &mut R) -> std::io::Result<BTreeMap<K, V>> {
        let i = decode_int(src)?;
        let mut dest = BTreeMap::new();

        for _ in 0..i as usize {
            let k = K::decode_bin(src)?;
            let v = V::decode_bin(src)?;
            dest.insert(k, v);
        }

        Ok(dest)
    }
}

impl<T: Type, U: Type> Type for Pair<T, U> {
    fn encode_bin<W: std::io::Write>(&self, dest: &mut W) -> std::io::Result<usize> {
        let mut n = self.0.encode_bin(dest)?;
        n += self.1.encode_bin(dest)?;
        Ok(n)
    }

    fn decode_bin<R: std::io::Read>(src: &mut R) -> std::io::Result<Self> {
        let a = T::decode_bin(src)?;
        let b = U::decode_bin(src)?;
        Ok((a, b))
    }
}

impl<T: Type, U: Type, V: Type> Type for Triple<T, U, V> {
    fn encode_bin<W: std::io::Write>(&self, dest: &mut W) -> std::io::Result<usize> {
        let mut n = self.0.encode_bin(dest)?;
        n += self.1.encode_bin(dest)?;
        n += self.2.encode_bin(dest)?;
        Ok(n)
    }

    fn decode_bin<R: std::io::Read>(src: &mut R) -> std::io::Result<Self> {
        let a = T::decode_bin(src)?;
        let b = U::decode_bin(src)?;
        let c = V::decode_bin(src)?;
        Ok((a, b, c))
    }
}

impl<T: Type> Type for Option<T> {
    fn encode_bin<W: std::io::Write>(&self, dest: &mut W) -> std::io::Result<usize> {
        match self {
            None => 0u8.encode_bin(dest),
            Some(x) => {
                let mut n = 255u8.encode_bin(dest)?;
                n += x.encode_bin(dest)?;
                Ok(n)
            }
        }
    }

    fn decode_bin<R: std::io::Read>(src: &mut R) -> std::io::Result<Self> {
        let i = u8::decode_bin(src)?;
        match i {
            0 => Ok(None),
            _ => T::decode_bin(src).map(Some),
        }
    }
}

impl<T: Type> Type for &T {
    fn encode_bin<W: std::io::Write>(&self, dest: &mut W) -> std::io::Result<usize> {
        Type::encode_bin(*self, dest)
    }

    fn decode_bin<R: std::io::Read>(_src: &mut R) -> std::io::Result<Self> {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Cannot decode into a reference",
        ))
    }
}

impl Type for &str {
    fn encode_bin<W: std::io::Write>(&self, dest: &mut W) -> std::io::Result<usize> {
        let i = self.len();
        let n = i.encode_bin(dest)?;
        dest.write_all(self.as_bytes())?;
        Ok(n + i)
    }

    fn decode_bin<R: std::io::Read>(_src: &mut R) -> std::io::Result<Self> {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Cannot decode into a str reference",
        ))
    }
}

impl<'a, T: Type> Type for Array<'a, T> {
    fn encode_bin<W: std::io::Write>(&self, dest: &mut W) -> std::io::Result<usize> {
        let i = self.len();
        i.encode_bin(dest)?;
        let mut n = 0;
        for x in self.as_ref().iter() {
            n += x.encode_bin(dest)?;
        }
        Ok(n)
    }

    fn decode_bin<R: std::io::Read>(src: &mut R) -> std::io::Result<Self> {
        let x = Vec::<T>::decode_bin(src)?;
        Ok(Array::Owned(x))
    }
}

impl<'a> Type for Str<'a> {
    fn encode_bin<W: std::io::Write>(&self, dest: &mut W) -> std::io::Result<usize> {
        let i = self.len();
        let n = i.encode_bin(dest)?;
        dest.write_all(self.as_ref().as_bytes())?;
        Ok(n + i)
    }

    fn decode_bin<R: std::io::Read>(src: &mut R) -> std::io::Result<Self> {
        let x = String::decode_bin(src)?;
        Ok(Str::Owned(x))
    }
}

impl<'a> Type for Bytes<'a> {
    fn encode_bin<W: std::io::Write>(&self, dest: &mut W) -> std::io::Result<usize> {
        let i = self.as_ref().len();
        let n = i.encode_bin(dest)?;
        dest.write_all(self.as_ref())?;
        Ok(n + i)
    }

    fn decode_bin<R: std::io::Read>(src: &mut R) -> std::io::Result<Self> {
        let x = Vec::<u8>::decode_bin(src)?;
        Ok(Bytes::Owned(x))
    }
}

#[cfg(test)]
mod tests {
    use crate::Type;

    mod irmin {
        pub use crate::Type;
    }

    #[test]
    fn test_int_string_pair() {
        let a = (123isize, "abc".to_string());
        let data = include_bytes!("../tests/int_string_pair.bin");
        let mut output = Vec::new();
        a.encode_bin(&mut output).unwrap();
        assert_eq!(output.as_slice(), data);

        let t: (isize, String) = Type::decode_bin(&mut output.as_slice()).unwrap();
        assert_eq!(a, t);
    }

    #[test]
    fn test_int_long_string_pair() {
        let s = [b'A'; 4096];
        let s = unsafe { std::str::from_utf8_unchecked(&s) };
        let a = (500isize, s.to_string());
        let data = include_bytes!("../tests/int_long_string_pair.bin");
        let mut output = Vec::new();
        a.encode_bin(&mut output).unwrap();
        assert_eq!(output.as_slice(), data);

        let t: (isize, String) = Type::decode_bin(&mut output.as_slice()).unwrap();
        assert_eq!(a, t);
    }

    #[test]
    fn test_struct1() {
        #[derive(Type, Debug, PartialEq)]
        struct Test {
            a: isize,
            b: Vec<String>,
        }

        let s = Test {
            a: 999,
            b: vec!["B".to_string(); 16],
        };
        let data = include_bytes!("../tests/struct1.bin");
        let mut output = Vec::new();
        s.encode_bin(&mut output).unwrap();
        assert_eq!(output.as_slice(), data);

        let t: Test = Type::decode_bin(&mut output.as_slice()).unwrap();
        assert_eq!(s, t);
    }

    #[test]
    fn test_enum1() {
        #[derive(Type, Debug, PartialEq)]
        enum Test {
            A(f64),
            B(Option<String>),
        }

        let s = (Test::A(4.5), Test::B(None));
        let data = include_bytes!("../tests/enum1.bin");
        let mut output = Vec::new();
        s.encode_bin(&mut output).unwrap();
        assert_eq!(output.as_slice(), data);

        let t: (Test, Test) = Type::decode_bin(&mut output.as_slice()).unwrap();
        assert_eq!(s, t);
    }
}
