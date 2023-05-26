use std::cmp::Ordering;
use core::ops;
use std::fmt;
use std::str::FromStr;



// copy and clone are reasonable as in rusttype::Point
#[derive(Copy, Clone, Debug, Eq)] 
pub struct Point{
    x:i32,
    y:i32,
}

impl Point{

    pub fn new(x:i32, y:i32) -> Point{
        Point{x:x, y:y}
    }

    pub fn mag(&self)->f32{
        return ((self.x*self.x + self.y*self.y)as f32).sqrt();
    }

    pub fn dot(&self, rhs:Point) -> i32{
        self.x*rhs.x + self.y*rhs.y
    }

    pub fn cross_z(p1:Point, p2:Point, p3:Point) -> i32{
        (p2.x-p1.x)*(p3.y-p1.y) - (p2.y-p1.y)*(p3.x-p1.x)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParsePointError;

impl FromStr for Point{
    type Err = ParsePointError;
    fn from_str(s: &str) -> Result<Self,Self::Err>{
        let (x, y) = s
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .and_then(|s| s.split_once(','))
            .ok_or(ParsePointError)?;
        let x_fromstr = x.parse::<i32>().map_err(|_| ParsePointError)?;
        let y_fromstr = y.parse::<i32>().map_err(|_| ParsePointError)?;
        Ok(Point{x:x_fromstr, y:y_fromstr})
    }
}

impl Ord for Point{
    fn cmp(&self, other: &Self) -> Ordering{
        self.y.cmp(&other.y).then(self.x.cmp(&other.x))
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl ops::Sub<Point> for Point{
    type Output = Point;
    fn sub(self, rhs: Point) -> Point{
        Point{x:self.x-rhs.x, y:self.y-rhs.y}
    }
}

impl ops::Add<Point> for Point{
    type Output = Point;
    fn add(self, rhs: Point) -> Point{
        Point{x:self.x+rhs.x, y:self.y+rhs.y}
    }
}

impl fmt::Display for Point{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f,"({},{})", self.x, self.y)
    }
}