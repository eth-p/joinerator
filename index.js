#!/usr/bin/env node
// ----------------------------------------------------------------------------
'use strict';
const clipboard = require('copy-paste');
const chalk = require('chalk');
const util = require('util');
const minimist = require('minimist');
const shuffle = require('knuth-shuffle').knuthShuffle;

const unicode = require('./unicode');
// ----------------------------------------------------------------------------

const ARGS = minimist(process.argv.slice(2), {
	'string': [
		'max',
		'above-count', 
		'above-frequency',
		'below-count',
		'below-frequency'
	],

	'alias':  {
		m: 'max',
		a: 'above-count',
		b: 'below-count',
		A: 'above-frequency',
		B: 'below-frequency'
	},

	'default': {
		'max':             'auto',
		'above-count':     '1',
		'below-count':     '1',
		'above-frequency': '60%',
		'below-frequency': '60%',
	}
});

// ----------------------------------------------------------------------------

/**
 * Parses a maximum/minimum number.
 * This supports percentages of the input length.
 */
function parse(input, arg) {
	if (typeof arg === 'number') return arg;
	if (arg.endsWith("%")) {
		return Math.ceil([...input].length * (parseFloat(arg.substring(0, arg.length - 1)) / 100));
	}

	return parseInt(arg, 10);
}


function randomize(input, limit, categories) {
	const chars    = [... input];
	const charsLen = chars.length;
	const charsLim = parse(chars, limit);
	const buckets  = new Array(chars.length);

	// Create default buckets.
	for (let i = 0; i < buckets.length; i++) {
		const bucket = buckets[i] = {};
		bucket.index = i;
		for (const [category, _] of Object.entries(categories)) {
			bucket[category] = 0;
		}
	}

	// Calculate passes and frequencies.
	const passes = Math.max(...Object.values(categories).map(x => parse(input, x.count)));
	let added = 0;
	for (const [category, data] of Object.entries(categories)) {
		data._pass_count = parse(input, data.count);
		data._pass_size  = parse(input, data.frequency);
		added += data._pass_count * data._pass_size;
	}

	const freqMod  = limit === 'auto' ? 1 : (
		(charsLim - charsLen) / added
	);


	// Calculate assignments.
	for (let pass = 0; pass < passes; pass++) {
		for (const [category, data] of Object.entries(categories)) {
			if (pass >= data._pass_count) continue;

			const overlay = new Array(buckets.length);
			overlay.fill(false, 0, overlay.length);
			overlay.fill(true, 0, Math.ceil(freqMod * data._pass_size));
			shuffle(overlay);

			for (let i = 0; i < overlay.length; i++) {
				if (overlay[i]) buckets[i][category]++;
			}
		}
	}

	// Generate resulting string.
	let remaining = limit === 'auto' ? Infinity : (charsLim - charsLen);
	let sb        = '';

	for (const bucket of buckets) {
		let char = chars[bucket.index];
		let sb   = char;

		if (remaining > 0 && char != ' ') {
			for (const [category, data] of Object.entries(categories)) {
				let count = Math.min(bucket[category], remaining);
				remaining -= count;

				for (let i = 0; i < count; i++) {
					sb += data.charset.random;
				}
			}
		}

		chars[bucket.index] = sb;
	}

	return chars.join('');
}


// ----------------------------------------------------------------------------
// Main:
//
const cb_get = util.promisify(clipboard.paste);
const cb_set = util.promisify(clipboard.copy);

function sleep(ms) {
    return new Promise((resolve, reject) => {
        setTimeout(resolve, ms);
    });
}

function transform(input) {
    return randomize(input, ARGS['max'], {
        ABOVE: {
            charset:   unicode.ABOVE,
            count:     ARGS['above-count'],
            frequency: ARGS['above-frequency']
        },
        BELOW: {
            charset:   unicode.BELOW,
            count:     ARGS['below-count'],
            frequency: ARGS['below-frequency']
        }
    });
}

async function main() {
    let last_input  = null;
    let last_output = null;
    let ignore      = true;
    
    while (true) {
        //await sleep(10);
        
        let input = await cb_get();
        //if (ignore) {
        //    last_output = input;
        //    ignore      = false;
        //    continue;
        //}
        
        if (input !== last_input && input !== last_output) {
            last_input  = input;
            last_output = transform(input);
            ignore = true;
            await cb_set(last_output);
            
            console.log(chalk.yellow("Input:"));
            console.log(input);
            console.log("\n");
        }

		break;
    }
}

main().then(() => {}, error => {
    console.error(error);
});

