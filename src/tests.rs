use super::*;
use crate::tests::Direction::{Left, Right};
use std::iter;

#[derive(Debug, Clone, Eq, PartialEq)]
struct SimpleTree {
    value: u8,
    left: Option<Box<SimpleTree>>,
    right: Option<Box<SimpleTree>>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Direction {
    Left,
    Right,
}

fn directions_from_bits(mut bits: usize, depth: u8) -> impl Iterator<Item = Direction> {
    let mut current_depth = 0u8;
    iter::from_fn(move || {
        if current_depth >= depth {
            None
        } else {
            current_depth += 1;
            let result = if bits % 2 == 0 { Left } else { Right };
            bits /= 2;
            Some(result)
        }
    })
}

impl SimpleTree {
    fn prune_or_increment(&mut self, it: impl IntoIterator<Item = Direction>) {
        let mut rope = Rope::new(self);
        let mut prune_direction = None;
        for dir in it {
            rope.advance_simul(|node| {
                let potential_prune_point = node.left.is_some() && node.right.is_some();
                let child = match dir {
                    Left => node.left.as_mut().unwrap().as_mut(),
                    Right => node.right.as_mut().unwrap().as_mut(),
                };
                if potential_prune_point && (child.left.is_none() || child.right.is_none()) {
                    prune_direction = Some(dir);
                    Simul::Advance(child)
                } else {
                    Simul::Hold(child)
                }
            })
        }
        let mut advance = true;
        let mut prune = true;
        while advance {
            rope.advance_map(|node| {
                match (&node.left, &node.right) {
                    (None, None) => {
                        advance = false;
                        return node;
                    }
                    (Some(_), Some(_)) => {
                        advance = false;
                        prune = false;
                        node.value += 1;
                        return node;
                    }
                    _ => (),
                };
                match (&mut node.left, &mut node.right) {
                    (Some(left), None) => left.as_mut(),
                    (None, Some(right)) => right.as_mut(),
                    _ => panic!("unreachable"),
                }
            })
        }
        match (prune, prune_direction) {
            (true, Some(Left)) => {
                rope.into_anchor().left = None;
            }
            (true, Some(Right)) => {
                rope.into_anchor().right = None;
            }
            _ => (),
        }
    }

    fn needs_prune(&mut self) -> bool {
        let mut node = self;
        loop {
            match (&node.left, &node.right) {
                (None, None) => return true,
                (Some(_), Some(_)) => {
                    node.value += 1;
                    return false;
                }
                _ => (),
            }
            match (&mut node.left, &mut node.right) {
                (Some(left), None) => node = left.as_mut(),
                (None, Some(right)) => node = right.as_mut(),
                _ => unreachable!(),
            }
        }
    }

    fn prune_or_increment_safe_impl(&mut self, mut it: impl Iterator<Item = Direction>) -> bool {
        match it.next() {
            None => return self.needs_prune(),
            Some(dir) => {
                let child = match dir {
                    Left => self.left.as_mut().unwrap().as_mut(),
                    Right => self.right.as_mut().unwrap().as_mut(),
                };
                if child.prune_or_increment_safe_impl(it) {
                    if self.left.is_some() && self.right.is_some() {
                        match dir {
                            Left => self.left = None,
                            Right => self.right = None,
                        }
                        false
                    } else {
                        true
                    }
                } else {
                    false
                }
            }
        }
    }

    fn prune_or_increment_safe(&mut self, it: impl IntoIterator<Item = Direction>) {
        self.prune_or_increment_safe_impl(it.into_iter());
    }
}

fn directions_example() -> impl Iterator<Item = Direction> {
    directions_from_bits(0b1101, 5)
}

#[test]
fn test_directions_from_bits() {
    assert_eq!(
        directions_example().collect::<Vec<_>>(),
        vec![Right, Left, Right, Right, Left]
    )
}

const fn empty_tree(value: u8) -> SimpleTree {
    SimpleTree {
        value,
        left: None,
        right: None,
    }
}

fn create_tree_example(final_branch: bool, final_value: u8) -> SimpleTree {
    SimpleTree {
        value: 0,
        left: Some(Box::new(empty_tree(1))),
        right: Some(Box::new(SimpleTree {
            value: 1,
            right: None,
            left: Some(Box::new(SimpleTree {
                value: 2,
                left: None,
                right: Some(Box::new(SimpleTree {
                    value: 3,
                    left: None,
                    right: Some(Box::new(SimpleTree {
                        value: 4,
                        right: None,
                        left: Some(Box::new(if final_branch {
                            SimpleTree {
                                value: final_value,
                                right: Some(Box::new(empty_tree(6))),
                                left: Some(Box::new(empty_tree(6))),
                            }
                        } else {
                            empty_tree(5)
                        })),
                    })),
                })),
            })),
        })),
    }
}

#[test]
fn test_prune() {
    let mut tree = create_tree_example(false, 5);
    tree.prune_or_increment(directions_example());
    assert_eq!(
        tree,
        SimpleTree {
            value: 0,
            left: Some(Box::new(empty_tree(1))),
            right: None,
        }
    );
}

#[test]
fn test_prune_safe() {
    let mut tree = create_tree_example(false, 5);
    tree.prune_or_increment_safe(directions_example());
    assert_eq!(
        tree,
        SimpleTree {
            value: 0,
            left: Some(Box::new(empty_tree(1))),
            right: None,
        }
    );
}

#[test]
fn test_increment() {
    let mut tree = create_tree_example(true, 5);
    tree.prune_or_increment(directions_example());
    assert_eq!(tree, create_tree_example(true, 6));
}

#[test]
fn test_increment_safe() {
    let mut tree = create_tree_example(true, 5);
    tree.prune_or_increment(directions_example());
    assert_eq!(tree, create_tree_example(true, 6));
}

#[test]
fn test_unsound() {
    // let mut x = 0;
    // let mut rope = Rope::new(&mut x);
    // let r1 = rope.get_lead();
    // let r2 = rope.get_lead_mut();
    // let s1 = format!("{r1}");
    // let s2 = format!("{r2}");
    // **r2 += 1;
    // let s3 = format!("{r1}");

    // Uncommenting the above should give a compiler error
}

#[test]
fn test_unsound_2() {
    // let mut x = [1; 4];
    // let (mut front, back) = x.split_at_mut(2);
    // let mut rope = Rope::new(back);
    // rope.advance_map(|back| {
    //     let parts = back.split_at_mut(1);
    //     front = parts.0;
    //     parts.1
    // });
    // let back = rope.into_anchor();
    // front[0] *= 2;
    // back[0] *= 2;
    // assert_eq!(x, [4, 1, 1, 1]);

    // Uncommenting above should give a compiler error about a reference escaping the closure.
}
