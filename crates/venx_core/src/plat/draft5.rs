#[derive(Debug)]
pub struct World {
    pub chunks: Vec<Chunk>,
    pub archichunks: Vec<ArchiChunk>,
    pub archistep: u16,
    pub loses: u32,
    pub merges: u32,
}

impl World {
    pub fn new(step: u16) -> Self {
        World {
            chunks: vec![],
            archichunks: vec![],
            archistep: step,
            loses: 0,
            merges: 0,
        }
    }
    pub fn add_chunk(&mut self, line: Vec<u32>) {
        let mut is_last = false;

        //

        if self.chunks.len() % (self.archistep as usize) == 0 {
            log::info!("Add Archi-Chunk");
            let a_chunk = ArchiChunk { line };
            self.archichunks.push(a_chunk);
            self.chunks.push(Chunk { line: vec![] });
            return;
        }

        let mut merges = 0;
        let mut loses = 0;

        let mut chunk: Vec<(u32, i32)> = vec![];
        for (a_bl, bl) in self.last_ach().line.iter().zip(line.iter()) {
            if a_bl == bl {
                merges += 1;
                if is_last {
                    chunk.last_mut().unwrap().0 += 1;
                } else {
                    is_last = true;
                    chunk.push((1, -1));
                }
            } else {
                loses += 1;
                if is_last {
                    is_last = false;
                    chunk.push((*bl, 1));
                } else {
                    if let Some(last) = chunk.last() {
                        if last.0 == *bl {
                            chunk.last_mut().unwrap().1 += 1;
                        }
                    } else {
                        chunk.push((*bl, 1));
                    }
                }
            }
        }
        let mut final_chunk = Chunk { line: vec![] };

        for (left, right) in chunk {
            final_chunk.line.push(left);
            if right != -1 {
                final_chunk.line.push(right as u32);
            }
        }

        self.chunks.push(final_chunk);

        self.merges += merges;
        self.loses += loses;
    }

    fn last_ach(&self) -> &ArchiChunk {
        self.archichunks.last().unwrap()
    }
}
#[derive(Debug, bitcode::Encode)]
pub struct Chunk {
    line: Vec<u32>,
}
#[derive(Debug)]
pub struct ArchiChunk {
    line: Vec<u32>,
}

#[test]
fn test_d5() {
    let ch = vec![5, 5, 5, 6, 7, 8, 9];
    let ch1 = vec![5, 55, 55, 6, 7, 1, 9];
    let ch2 = vec![5, 5, 5, 6, 1, 1, 1];

    let mut w = World::new(10);

    w.add_chunk(ch);
    w.add_chunk(ch1);
    w.add_chunk(ch2);

    dbg!(w);
}
