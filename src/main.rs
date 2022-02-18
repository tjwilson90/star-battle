use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use bstr::ByteSlice;
use disjoint_sets::UnionFind;
use regex::bytes::Regex;
use z3::{Config, Context, SatResult, Solver};
use z3::ast::Bool;

struct Walls(u8);

impl Walls {
    fn new() -> Self {
        Self(0)
    }

    fn set_left(&mut self) {
        self.0 |= 1;
    }

    fn left(&self) -> bool {
        self.0 & 1 != 0
    }

    fn set_top(&mut self) {
        self.0 |= 2;
    }

    fn top(&self) -> bool {
        self.0 & 2 != 0
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut file = File::open("/Users/twilson/code/star-battle/star-battle.html")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let cell_regex = Regex::new(r#"<div tabindex="1" class="cell selectable(.*?)cell-off".*?</div>"#)?;

    let mut walls = Vec::new();
    for cap in cell_regex.captures_iter(&buf) {
        let mut cell_walls = Walls::new();
        if cap[1].find("bt").is_some() {
            cell_walls.set_top();
        }
        if cap[1].find("bl").is_some() {
            cell_walls.set_left();
        }
        walls.push(cell_walls);
    }
    let mut uf = UnionFind::new(walls.len());
    let size = (walls.len() as f64).sqrt().round() as usize;

    for i in 0..size {
        for j in 0..size {
            let idx = size * i + j;
            let w = &walls[idx];
            if i > 0 && !w.top() {
                uf.union(idx, size * (i - 1) + j);
            }
            if j > 0 && !w.left() {
                uf.union(idx, size * i + j - 1);
            }
        }
    }
    let mut groups = HashMap::<_, Vec<_>>::new();
    for i in 0..size {
        for j in 0..size {
            let idx = size * i + j;
            let group = uf.find(idx);
            groups.entry(group).or_default().push(idx);
        }
    }

    let ctx = &Context::new(&Config::new());
    let solver = Solver::new(ctx);
    let mut cell_vars = Vec::new();
    for i in 0..size {
        for j in 0..size {
            cell_vars.push(Bool::new_const(ctx, format!("cell_{}_{}", i, j)));
        }
    }
    for i in 0..size {
        for j in 0..size {
            let var = &cell_vars[size * i + j];
            if i > 0 {
                solver.assert(&Bool::and(ctx, &[var, &cell_vars[size * (i - 1) + j]]).not());
            }
            if j > 0 {
                solver.assert(&Bool::and(ctx, &[var, &cell_vars[size * i + j - 1]]).not());
            }
            if i > 0 && j > 0 {
                solver.assert(&Bool::and(ctx, &[var, &cell_vars[size * (i - 1) + j - 1]]).not());
            }
            if i > 0 && j < size - 1 {
                solver.assert(&Bool::and(ctx, &[var, &cell_vars[size * (i - 1) + j + 1]]).not());
            }
        }
    }
    let num_stars = if size < 10 {
        1
    } else if size < 14 {
        2
    } else if size < 17 {
        3
    } else if size < 21 {
        4
    } else if size < 25 {
        5
    } else {
        6
    };
    for i in 0..size {
        let mut constraint = Vec::new();
        for j in 0..size {
            let var = &cell_vars[size * i + j];
            constraint.push((var, 1));
        }
        solver.assert(&Bool::pb_eq(ctx, &constraint, num_stars));
    }
    for j in 0..size {
        let mut constraint = Vec::new();
        for i in 0..size {
            let var = &cell_vars[size * i + j];
            constraint.push((var, 1));
        }
        solver.assert(&Bool::pb_eq(ctx, &constraint, num_stars));
    }
    for group in groups.values() {
        let mut constraint = Vec::new();
        for idx in group {
            let var = &cell_vars[*idx];
            constraint.push((var, 1));
        }
        solver.assert(&Bool::pb_eq(ctx, &constraint, num_stars));
    }
    assert_eq!(solver.check(), SatResult::Sat);
    let model = solver.get_model().unwrap();

    print!("┌───");
    for _ in 1..size {
        print!("┬───")
    }
    println!("┐");
    for i in 0..size {
        for j in 0..size {
            let cell = model.eval(&cell_vars[size * i + j], false).unwrap().as_bool().unwrap();
            if cell {
                print!("│ ★ ");
            } else {
                print!("│   ");
            }
        }
        println!("│");
        if i != size - 1 {
            print!("├───");
            for _ in 1..size {
                print!("┼───")
            }
            println!("┤");
        }
    }
    print!("└───");
    for _ in 1..size {
        print!("┴───")
    }
    println!("┘");
    Ok(())
}
