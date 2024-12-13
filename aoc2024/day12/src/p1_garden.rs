use crate::p1_regions::Region;
use itertools::Itertools;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[allow(unused_imports)]
use winnow::{
    ascii::*,
    combinator::*,
    error::*,
    prelude::*,
    stream::*,
    token::*,
    {PResult, Parser},
};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Position(pub usize);
impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Position {
    fn relative_to<'a>(&'a self, garden: &'a Garden) -> RelativePosition<'a> {
        RelativePosition { pos: self, garden }
    }
}

pub struct RelativePosition<'a> {
    pub pos: &'a Position,
    pub garden: &'a Garden,
}

#[expect(clippy::needless_lifetimes)]
impl<'a> Display for RelativePosition<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}x,{}y)",
            //            self.pos,
            self.pos.0 % self.garden.width,
            self.pos.0 / self.garden.width
        )
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Flower(pub char);
impl Display for Flower {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct Garden {
    pub inner: Vec<Flower>,
    pub width: usize,
}

impl Display for Garden {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // we want to print on multiple lines, except for on the last line
        let together = |line: &[Flower]| line.iter().map(|a| a.0).join("");
        let mut lines_reversed = self.inner.chunks_exact(self.width).rev();
        let last_line = lines_reversed.next().expect("non-empty");
        for line in lines_reversed.rev() {
            writeln!(f, "{}", together(line))?;
        }
        write!(f, "{}", together(last_line))
    }
}

impl Garden {
    pub fn get_regions(&self) -> Vec<Region> {
        /// Recursive exploration of garden map.
        /// Go every cardinal direction, if the spot is valid and new.
        /// If nowhere to go, propagate up a collection for the valid flower.
        /// This flower connects a region, since recursive must be touching.
        /// If the flower on the upper level is different,
        /// it means an entire regions' flowers was found.
        /// This is stored, such that every one of these is unique.
        /// A vector is used because no key can be used to uniquely identify these,
        /// except an arbitrary position value.
        fn explore(
            garden: &Garden,
            this_pos: Position,
            visited: &mut HashSet<Position>,
        ) -> (Region, Vec<Region>) {
            let this_flower = *garden
                .inner
                .get(this_pos.0)
                .expect("only call this function with a valid position");
            //            eprintln!(
            //                "Looking through {} at position {}",
            //                this_flower,
            //                this_pos.relative_to(garden)
            //            );

            let (bordering_regions, others): (Vec<Region>, Vec<Vec<Region>>) = garden
                .bordering_pos(&this_pos)
                .filter(|new_pos| garden.has_pos(new_pos))
                .filter_map(|valid_pos| {
                    if visited.insert(valid_pos) {
                        // new position, explore branch
                        Some(explore(garden, valid_pos, visited))
                    } else {
                        // positions was already seen
                        None
                    }
                })
                .unzip();

            bordering_regions.into_iter().fold(
                (
                    Region {
                        flower: this_flower,
                        positions: HashSet::from([this_pos]),
                    },
                    others.into_iter().flatten().collect::<Vec<_>>(),
                ),
                |(mut self_similar, mut others), bordering_region| {
                    if bordering_region.flower == self_similar.flower {
                        eprintln!(
                            "Connecting {} of {} flowers at {} to {}.",
                            bordering_region.positions.len(),
                            bordering_region.flower,
                            bordering_region
                                .positions
                                .iter()
                                .map(|pos| pos.relative_to(garden))
                                .join(", "),
                            this_pos.relative_to(garden)
                        );
                        let prev = self_similar.clone();
                        self_similar.positions.extend(bordering_region.positions);
                        eprintln!("{}", prev.changing_to(&self_similar, garden));
                    } else {
                        others.push(bordering_region);
                    }

                    (self_similar, others)
                },
            )
        }
        
        let (first, mut rest) = explore(self, Position(0), &mut HashSet::from([Position(0)]));
        eprintln!("{}", first.relative_to(self));
        rest.push(first);
        rest
    }
    pub fn bordering_pos(&self, this_pos: &Position) -> impl Iterator<Item = Position> {
        [
            // left
            this_pos.0.checked_sub(1),
            // right
            this_pos.0.checked_add(1),
            // up
            this_pos.0.checked_sub(self.width),
            // down
            this_pos.0.checked_add(self.width),
        ]
        .into_iter()
        .flatten()
        .map(Position)
    }
    pub fn has_pos(&self, pos: &Position) -> bool {
        self.inner.len() > pos.0
    }
}
impl FromStr for Garden {
    type Err = ErrMode<ContextError>;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        parse_garden.parse_next(&mut s)
    }
}

fn parse_garden(input: &mut &str) -> PResult<Garden> {
    separated(1.., parse_line, line_ending)
        // get len of first element
        .verify_map(|v: Vec<Vec<_>>| v.first().map(|v_1| v_1.len()).map(|v_1_len| (v, v_1_len)))
        // ensure all lens are the same
        .verify(|(v, _)| v.iter().map(|v_i| v_i.len()).all_equal())
        .map(|(v, width): (Vec<Vec<_>>, _)| {
            let inner = v.into_iter().flat_map(|v_i| v_i.into_iter()).collect();
            Garden { inner, width }
        })
        .parse_next(input)
}

fn parse_line(input: &mut &str) -> PResult<Vec<Flower>> {
    repeat(1.., any.verify(|c| AsChar::is_alpha(c)).map(Flower)).parse_next(input)
}

#[cfg(test)]
mod tests {
    use crate::p1_garden::{Flower, Garden};

    #[test]
    fn garden_parses() {
        let garden = include_str!("EXAMPLE").parse::<Garden>().unwrap();
        let flowers = [
            'A', 'A', 'A', 'A', 'B', 'B', 'C', 'D', 'B', 'B', 'C', 'C', 'E', 'E', 'E', 'C',
        ]
        .map(Flower);
        assert_eq!(garden.inner, flowers, "Flowers should parse as expected.");
    }
}