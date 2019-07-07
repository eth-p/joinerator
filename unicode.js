'use strict';

const ABOVE = module.exports.ABOVE = [
	"\u0300", "\u0301", "\u0302", "\u0303",
	"\u0304", "\u0305", "\u0306", "\u0307",
	"\u0308", "\u0309", "\u030A", "\u030B",
	"\u030C", "\u030D", "\u030E", "\u030F",
	"\u0310", "\u0311", "\u0312", "\u0313",
	"\u0314", "\u0315", "\u031A", "\u031B",
	"\u033D", "\u033E", "\u033F", "\u0340",
	"\u0341", "\u0342", "\u0343", "\u0344",
	"\u0346", "\u034A", "\u034B", "\u034C",
	"\u0350", "\u0351", "\u0352", "\u0357",
	"\u0358", "\u035B", "\u0363", "\u0364",
	"\u0365", "\u0366", "\u0367", "\u0368",
	"\u0369", "\u036A", "\u036B", "\u036C",
	"\u036D", "\u036E", "\u036F", "\u031A",
];

const BELOW = module.exports.BELOW = [
	"\u0316", "\u0317", "\u0318", "\u0319",
	"\u031C", "\u031D", "\u031E", 
	"\u031F", "\u0320", "\u0321", "\u0323",
	"\u0324", "\u0325", "\u0326", "\u0327",
	"\u0328", "\u0329", "\u032A", "\u032B",
	"\u032C", "\u032D", "\u032E", "\u032F",
	"\u0330", "\u0331", "\u0332", "\u0333",
	"\u0339", "\u033A", "\u033B", "\u033C",
	"\u0345", "\u0347", "\u0348", "\u0349",
	"\u034D", "\u034E", "\u0353", "\u0354",
	"\u0355", "\u0356", "\u0359", "\u035A",
];

const OVERLAY = module.exports.OVERLAY = [
	"\u0334", "\u0335", "\u0336", "\u0337",
	"\u0338"
];

const JOINER = module.exports.JOINER = [
	"\u035C", "\u035D", "\u035E", "\u035F",
	"\u0360", "\u0361", "\u0362"
];

// Create random getter on each array.
for (const arr of [ABOVE, BELOW, OVERLAY, JOINER]) {
	Object.defineProperty(arr, 'random', {
		get: () => arr[Math.floor(Math.random() * arr.length)]
	});
}

