// -------------------------------------------------------------------------------------------------
// joinerator | Copyright (C) 2019 eth-p
// -------------------------------------------------------------------------------------------------
use std::cell::Cell;
use std::collections::HashMap;

use crate::repertoire::{Glyph, GlyphPosition, Repertoire};
use core::borrow::BorrowMut;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::cmp::min;
// -------------------------------------------------------------------------------------------------

pub struct Joinerator<'a> {
    pub options: Options<'a>,
    glyphs: HashMap<GlyphPosition, Vec<Glyph>>,
    rng: ThreadRng,
}

pub struct Options<'a> {
    pub allow_unreadable: bool,
    pub limit: Option<usize>,

    pub repertoire: &'a Repertoire,
    pub generator: Vec<GeneratorOptions>,
}

pub struct GeneratorOptions {
    pub category: GlyphPosition,
    pub frequency: GeneratorFrequency,
    pub stacking: usize,
}

pub enum GeneratorFrequency {
    Percentage(f32),
    Fixed(usize),
}

// -------------------------------------------------------------------------------------------------

impl<'a> Joinerator<'a> {
    pub fn new(options: Options<'a>) -> Self {
        let mut glyphs = HashMap::new();
        for category in options.generator.iter().map(|g| g.category) {
            glyphs.insert(
                category,
                options
                    .repertoire
                    .glyphs
                    .iter()
                    .filter(|g| g.position == category)
                    .cloned()
                    .collect(),
            );
        }

        Joinerator {
            options,
            glyphs,
            rng: thread_rng(),
        }
    }

    /// Processes a string, adding Unicode combining characters to it.
    ///
    /// ## Arguments
    /// - `input - The input string.
    ///
    /// ## Returns
    /// A copy of the original string with extra Unicode combining characters added to it.
    /// The frequency and distribution of the combining characters is based on the options given
    /// to the Joinerator when it was initialized.
    pub fn process(&mut self, input: &str) -> String {
        let input_len = input.chars().count();
        let mut bucket = self.create_bucket(input);
        let mut passes = self.create_passes(input);
        let mut pass_buffer: Vec<bool> = Vec::with_capacity(input_len);
        pass_buffer.resize(input_len, false);

        if self.options.limit.is_some() && self.options.limit.unwrap() <= input_len {
            return input.to_owned();
        }

        let (mut remaining, frequency_modifier) = if self.options.limit.is_some() {
            let limit = self.options.limit.unwrap();
            (
                limit - input_len,
                ((limit - input_len) as f32) / (passes.total_additions as f32),
            )
        } else {
            (usize::max_value(), 1.0)
        };

        // Execute passes.
        for _pass in 0..passes.total_iterations {
            for (_category, pass_descriptor) in passes.descriptors.borrow_mut() {
                self.run_pass(
                    &mut bucket,
                    pass_descriptor,
                    &mut pass_buffer,
                    frequency_modifier,
                )
            }
        }

        // Apply passes.
        //  -> For every character
        //    -> For every category (glyph position)
        //       -> Add an applicable combining glyph.
        bucket.chars.shuffle(&mut self.rng);
        for char in bucket.chars.iter_mut() {
            char.combined.push(char.primary_glyph);

            for (category, c) in char.additional_glyphs.iter() {
                if c.get() < 1 {
                    continue;
                }

                let count = if self.options.limit.is_some() {
                    min(c.get(), remaining)
                } else {
                    c.get()
                };

                // Determine what combining glyphs can be applied to the primary glyph.
                let category_glyphs = self.glyphs.get(category).unwrap();
                let applicable_glyphs: Vec<&Glyph> = if self.options.allow_unreadable {
                    category_glyphs.iter().collect()
                } else {
                    category_glyphs
                        .iter()
                        .filter(|g| g.is_applicable(char.primary_glyph))
                        .collect()
                };

                if applicable_glyphs.len() < 1 {
                    continue;
                }

                // Add the combining glyphs to the primary glyph.
                for _ in 0..count {
                    char.combined
                        .push(applicable_glyphs.choose(&mut self.rng).unwrap().codepoint)
                }

                remaining -= count;
            }
        }

        // Return the combined string.
        bucket.chars.sort_by(|a, b| a.position.cmp(&b.position));
        bucket.chars.into_iter().map(|c| c.combined).collect()
    }

    /// Creates the bucket used to hold information about each character in the input string.
    ///
    /// ## Arguments
    /// - `input` - The input string.
    ///
    /// ## Returns
    /// A struct containing a vector which holds data related to each character in the input string.
    fn create_bucket(&self, input: &str) -> Bucket {
        let mut bucket = Bucket::new();

        for (index, char) in input.chars().enumerate() {
            let mut item = BucketItem::new(index, char);
            for category in self.options.generator.iter().map(|g| g.category) {
                item.additional_glyphs.insert(category, Cell::new(0));
            }

            bucket.chars.push(item);
        }

        bucket
    }

    /// Creates and calculates information about each of the passes to be run against the input string.
    ///
    /// ## Arguments
    /// - `input` - The input string.
    ///
    /// ## Returns
    /// A struct containing information and metadata about the generator passes that will be run.
    fn create_passes(&self, input: &str) -> Passes {
        let mut descriptors: HashMap<GlyphPosition, PassDescriptor> = HashMap::new();
        let mut additions: usize = 0;

        for generator in self.options.generator.iter() {
            let chars = match generator.frequency {
                GeneratorFrequency::Fixed(n) => n,
                GeneratorFrequency::Percentage(p) => (p * (input.chars().count() as f32)) as usize,
            };

            additions += chars * generator.stacking;
            descriptors.insert(
                generator.category,
                PassDescriptor {
                    category: generator.category,
                    passes: generator.stacking,
                    chars,
                },
            );
        }

        Passes {
            descriptors,
            total_additions: additions,
            total_iterations: self
                .options
                .generator
                .iter()
                .map(|v| v.stacking)
                .max()
                .unwrap(),
        }
    }

    /// Executes a pass of the generator for a specific category.
    /// This should be done for each category.
    ///
    /// ## Arguments
    /// - `bucket`     - The bucket created by `create_bucket`
    /// - `descriptor` - A pass descriptor created by `create_passes`
    /// - `buffer`     - A vector of booleans the size of the input string.
    ///                  This is intended to be reused.
    /// - `modifier`   - The frequency modifier (used for limiting)
    fn run_pass(
        &mut self,
        bucket: &mut Bucket,
        descriptor: &mut PassDescriptor,
        buffer: &mut Vec<bool>,
        modifier: f32,
    ) {
        if descriptor.passes <= 0 {
            return;
        }

        descriptor.passes -= 1;

        // Fill the buffer.
        let bound = ((descriptor.chars as f32) * modifier) as usize;
        for i in 0..buffer.len() {
            buffer[i] = if i < bound { true } else { false }
        }

        // Shuffle the buffer.
        buffer.shuffle(&mut self.rng);

        // Update the bucket items with the results of the shuffle.
        for char in bucket.chars.iter_mut() {
            if buffer[char.position] {
                let num = char
                    .additional_glyphs
                    .get_mut(&descriptor.category)
                    .unwrap();
                num.replace(num.get() + 1);
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

struct Bucket {
    chars: Vec<BucketItem>,
}

struct BucketItem {
    additional_glyphs: HashMap<GlyphPosition, Cell<usize>>,
    primary_glyph: char,
    combined: String,
    position: usize,
}

struct Passes {
    total_iterations: usize,
    total_additions: usize,
    descriptors: HashMap<GlyphPosition, PassDescriptor>,
}

struct PassDescriptor {
    category: GlyphPosition,
    passes: usize,
    chars: usize,
}

impl Bucket {
    pub fn new() -> Self {
        Bucket { chars: Vec::new() }
    }
}

impl BucketItem {
    pub fn new(index: usize, glyph: char) -> Self {
        BucketItem {
            additional_glyphs: HashMap::new(),
            primary_glyph: glyph,
            position: index,
            combined: String::new(),
        }
    }
}
