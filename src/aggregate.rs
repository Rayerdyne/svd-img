use image::{
    Rgb, Rgba,
};

pub trait Aggregator {
    fn i32_from_rgb(&self, p: Rgb<u8>) -> i32;
    fn i32_from_rgba(&self, p: Rgba<u8>) -> i32;

    fn rgb_from_i32(&self, x: i32) -> Rgb<u8>;
    fn rgba_from_i32(&self, x: i32) -> Rgba<u8>;
}
pub struct Aggregator1;
pub struct Aggregator2;

impl Aggregator for Aggregator1 {
    fn i32_from_rgba(&self, p: Rgba<u8>) -> i32 {
        let mut r = 0_i32;
        for i in 0..8 {
            let x = 1 << i;
            let i4 = 4 * i;
            let y = 1 << i4;
            r |= if p[0] & x != 0 { y << 3 } else { 0 };
            r |= if p[1] & x != 0 { y << 2 } else { 0 };
            r |= if p[2] & x != 0 { y << 1 } else { 0 };
            r |= if p[3] & x != 0 { y      } else { 0 };
        }
        r
    }

    fn i32_from_rgb(&self, p: Rgb<u8>) -> i32 {
        let mut r = 0_i32;
        for i in 0..8 {
            let x = 1 << i;
            let i3 = 3 * i;
            let y = 1 << i3 as i32;
            r |= if p[0] & x != 0 { y << 2 } else { 0 };
            r |= if p[1] & x != 0 { y << 1 } else { 0 };
            r |= if p[2] & x != 0 { y      } else { 0 };
        }
        r
    }

    fn rgba_from_i32(&self, p: i32) -> Rgba<u8> {
        let mut r = 0_u8; let mut g = 0_u8; let mut b = 0_u8; let mut a = 0_u8;
        let mut t = p;
        for i in 0..8 {
            let x = 1 << i as u8;
            r |= if (t & 8) != 0 { x } else { 0 };
            g |= if (t & 4) != 0 { x } else { 0 };
            b |= if (t & 2) != 0 { x } else { 0 };
            a |= if (t & 1) != 0 { x } else { 0 };
            t >>= 4;
        }
        Rgba([r, g, b, a])
    }

    fn rgb_from_i32(&self, p: i32) -> Rgb<u8> {
        let mut r = 0_u8; let mut g = 0_u8; let mut b = 0_u8; 
        let mut t = p;
        for i in 0..8 {
            let x = 1 << i as u8;
            r |= if (t & 4) != 0 { x } else { 0 };
            g |= if (t & 2) != 0 { x } else { 0 };
            b |= if (t & 1) != 0 { x } else { 0 };
            t >>= 3;
        }
        Rgb([r, g, b])
    }
}

impl Aggregator for Aggregator2 {
    fn i32_from_rgb(&self, p: Rgb<u8>) -> i32 {
        return (p[0] as i32) << 16_i32 +
               (p[1] as i32) << 8_i32  + 
                p[0] as i32
    }

    fn i32_from_rgba(&self, p: Rgba<u8>) -> i32 {
        return (p[3] as i32) << 24_i32 +
               (p[2] as i32) << 16_i32 +
               (p[1] as i32) << 8_i32  + 
                p[0] as i32
    }

    fn rgb_from_i32(&self, x: i32) -> Rgb<u8> {
        return Rgb([
            (x >> 16) as u8 & 0xff as u8,
            (x >> 8) as u8 & 0xff as u8,
            x as u8 & 0xff as u8,
        ])
    }

    fn rgba_from_i32(&self, x: i32) -> Rgba<u8> {
        return Rgba([
            (x >> 24) as u8 & 0xff as u8,
            (x >> 16) as u8 & 0xff as u8,
            (x >> 8) as u8 & 0xff as u8,
            x as u8 & 0xff as u8,
        ])
    }
}

#[allow(dead_code, unused_imports)]
mod tests {
    
    use super::Aggregator;
    use super::Aggregator1;

    fn test_jspity(n: i32, aggregator: Box<dyn Aggregator>) {
        let p = aggregator.rgba_from_i32(n);
        println!("pixel: {:?}", p);
        let r = aggregator.i32_from_rgba(p);
        println!("res: {:b}", r);
        println!("of:  {:b}", n);
        assert_eq!(r, n);
    }

    #[test]
    fn test_jspity2() {
        let mut i = 256;
        while i < 4096 {
            test_jspity(i, Box::new(Aggregator1));
            i += 256;
        }
    }
}