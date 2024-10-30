use crate::alphabet::{alphabet_cardinality, ascii_to_encoded, ascii_to_index};
use crate::simd_instructions::{AlignedVectorArray, SimdVec256};
use alphabet::Alphabet;

pub const NUM_NUCLEOTIDE_MILESTONES: usize = 8; //4 nucs, plus N, and $, padded to 8
pub const NUM_AMINO_MILESTONES: usize = 24; //20 AAs, plus X, and $, padded to 24
pub const NUM_NUCLEOTIDE_BIT_VECTORS: usize = 3;
pub const NUM_AMINO_BIT_VECTORS: usize = 5;
pub const NUM_POSITIONS_PER_BLOCK: usize = 256;

#[derive(Clone)]
pub struct NucleotideBwtBlock {
    milestones: [u64; NUM_NUCLEOTIDE_MILESTONES],
    bit_vectors: [SimdVec256; NUM_NUCLEOTIDE_BIT_VECTORS],
}

#[derive(Clone)]
pub struct AminoBwtBlock {
    milestones: [u64; NUM_AMINO_MILESTONES],
    bit_vectors: [SimdVec256; NUM_AMINO_BIT_VECTORS],
}

impl NucleotideBwtBlock {
    pub fn new() -> Self {
        NucleotideBwtBlock {
            milestones: [0; NUM_NUCLEOTIDE_MILESTONES],
            bit_vectors: [SimdVec256::zero(); NUM_NUCLEOTIDE_BIT_VECTORS],
        }
    }

    #[inline]
    pub fn set_milestones(&mut self, values: &Vec<u64>) {
        for milestone_idx in 0..NUM_NUCLEOTIDE_MILESTONES {
            self.milestones[milestone_idx] = values[milestone_idx];
        }
    }

    #[inline]
    pub fn get_milestone(&self, letter_idx: u8) -> u64 {
        return self.milestones[letter_idx as usize];
    }

    #[inline]
    pub fn global_occurrence(&self, local_query_position: usize, letter_idx: u8) -> u64 {
        let milestone_count = self.get_milestone(letter_idx);
        let vectors = &self.bit_vectors;
        let occurrence_vector = match letter_idx {
            0 => vectors[1].and(&vectors[2]), //A:    0b110
            1 => vectors[0].and(&vectors[2]), //C:    0b101
            2 => vectors[0].and(&vectors[1]), //G:    0b011
            3 => vectors[2].andnot(&vectors[1].andnot(&vectors[0])), //T:    0b001
            4 => vectors[2].andnot(&vectors[0].andnot(&vectors[1])), //N:    0b010
            _ => {
                panic!("illegal letter index given in global occurrence function");
            } //assume every other character is an N, since it's illegal to search for a sentinel
        };

        let popcount = occurrence_vector.masked_popcount(local_query_position);

        //generate the position bitmask

        return popcount as u64 + milestone_count;
        todo!("check to make sure the occurrence calculations are correct!");
    }
}

impl AminoBwtBlock {
    pub fn new() -> Self {
        AminoBwtBlock {
            milestones: [0; NUM_AMINO_MILESTONES],
            bit_vectors: [SimdVec256::zero(); NUM_AMINO_BIT_VECTORS],
        }
    }
    #[inline]
    pub fn set_milestones(&mut self, values: &Vec<u64>) {
        for milestone_idx in 0..NUM_AMINO_MILESTONES {
            self.milestones[milestone_idx] = values[milestone_idx];
        }
    }
    #[inline]
    pub fn get_milestone(&self, letter_idx: u8) -> u64 {
        return self.milestones[letter_idx as usize];
    }
    #[inline]
    pub fn global_occurrence(&self, local_query_position: usize, letter_idx: u8) -> usize {
        let milestone_count = self.get_milestone(letter_idx);
        let vecs = &self.bit_vectors;
        let occurrence_vector = match letter_idx {
            0 => vecs[3].and(&vecs[4].andnot(&vecs[2])), //A:    0b01100
            1 => vecs[3].andnot(&vecs[2]).and(&vecs[1].andnot(&vecs[0])), //C:    0b10111
            2 => vecs[1].and(&vecs[4].andnot(&vecs[0])), //D:    0b00011
            3 => vecs[4].andnot(&vecs[2].and(&vecs[1])), //E: 0b00110
            4 => vecs[0].andnot(&vecs[3]).and(&vecs[2].and(&vecs[1])), //F:    0b11110
            5 => vecs[2].andnot(&vecs[0].andnot(&vecs[4])), //G:    0b11010
            6 => vecs[2].andnot(&vecs[3]).and(&vecs[1].and(&vecs[0])), //H: 0b11011
            7 => vecs[2].andnot(&vecs[1].andnot(&vecs[4])), //I:    0b11001
            8 => vecs[3].andnot(&vecs[1].andnot(&vecs[4])), //K:    0b10101
            9 => vecs[1].andnot(&vecs[0].andnot(&vecs[4])), //L:    0b11100
            10 => vecs[1].andnot(&vecs[3]).and(&vecs[2].and(&vecs[0])), //M:    0b11101
            11 => vecs[0].or(&vecs[1]).andnot(&vecs[2].andnot(&vecs[3])), //N:    0b01000
            12 => vecs[3].and(&vecs[4].andnot(&vecs[0])), //P:    0b01001,
            13 => vecs[3].or(&vecs[1]).andnot(&vecs[0].andnot(&vecs[2])), //Q:    0b00100
            14 => vecs[3].andnot(&vecs[2].andnot(&vecs[4])), //R:    0b10011
            15 => vecs[3].and(&vecs[4].andnot(&vecs[1])), //S:    0b01010
            16 => vecs[2].and(&vecs[4].andnot(&vecs[0])), //T:    0b00101
            17 => vecs[3].andnot(&vecs[0].andnot(&vecs[4])), //V:    0b10110
            18 => vecs[3].or(&vecs[2]).andnot(&vecs[1].andnot(&vecs[0])), //W:    0b00001
            19 => vecs[0].or(&vecs[2]).andnot(&vecs[3].andnot(&vecs[1])), //Y:    0b00010
            20 => vecs[3].and(&vecs[2]).and(&vecs[1].and(&vecs[0])), //Ambiguity character:  0b11111
            // 0b00000 is sentinel, but since you can't search for sentinels, it is not included here.
            _ => {
                panic!("illegal letter index given in global occurrence function");
            }
        };

        let popcount = occurrence_vector.masked_popcount(local_query_position);
        return popcount as usize + milestone_count as usize;
        todo!("check to make sure the occurrence calculations are correct!");
    }
}

pub enum Bwt {
    Nucleotide(Vec<NucleotideBwtBlock>),
    Amino(Vec<AminoBwtBlock>),
}

impl Bwt {
    /// sets a single symbol at the given position in the bwt bit vectors.
    /// this function is meant to be run for every position in the bwt
    /// as a part of BWT data creation
    pub fn set_symbol_at(&self, btw_position: usize, mut encoded_symbol: u8) {
        //find the block, byte, and bit of the data we're setting
        let position_block_idx: usize = (btw_position / NUM_POSITIONS_PER_BLOCK) as usize;
        let position_in_block = btw_position % NUM_POSITIONS_PER_BLOCK;

        //create a bitmask, we'll use this to set the bit with an OR operation
        let vector_bitmask = SimdVec256::as_one_hot(position_in_block);

        let mut bit_vector_idx = 0;
        while encoded_symbol != 0 {
            if encoded_symbol & 0x1 == 0 {
                match self {
                    Bwt::Nucleotide(vec) => {
                        vec[position_block_idx].bit_vectors[bit_vector_idx].or(&vector_bitmask);
                    }
                    Bwt::Amino(vec) => {
                        vec[position_block_idx].bit_vectors[bit_vector_idx].or(&vector_bitmask);
                    }
                }
            }
            encoded_symbol >>= 1;
            bit_vector_idx += 1;
        }
    }

    pub fn set_milestones(&mut self, block_idx: usize, counts: &Vec<u64>) {
        match self {
            Bwt::Nucleotide(vec) => vec[block_idx].set_milestones(counts),
            Bwt::Amino(vec) => vec[block_idx].set_milestones(counts),
        }
    }
    fn get_milestone(&self, block_idx: usize, letter_idx: u8) -> u64 {
        return match self {
            Bwt::Nucleotide(vec) => vec[block_idx].milestones[letter_idx as usize],
            Bwt::Amino(vec) => vec[block_idx].milestones[letter_idx as usize],
        };
    }
    pub fn global_occurrence(&self, pointer_global_position: usize, letter_idx: u8) -> usize {
        let block_idx: usize = pointer_global_position / NUM_POSITIONS_PER_BLOCK;
        let local_query_position: usize = pointer_global_position % NUM_POSITIONS_PER_BLOCK;

        match self {
            Bwt::Nucleotide(vec) => {
                vec[block_idx].global_occurrence(local_query_position, letter_idx)
            }
            Bwt::Amino(vec) => vec[block_idx].global_occurrence(local_query_position, letter_idx),
        }
    }
}
