use criterion::{black_box, criterion_group, criterion_main, Criterion};
use schedulib::single_machine::*;
use schedulib::jobs::*;


fn example_200_a() -> (Vec<Time>, Vec<Time>, Vec<Time>) {
		(
			// 200 jobs with randomly chosen times
			// processing times:
			vec![  13,   17,    6,    2,   16,   29,   22,   24,   25,   11,   27,   16,   25,   25,    5,   16,   24,   13,    4,   29,   24,   13,    3,   19,    5,    4,   12,    4,   27,   23,   22,   20,   19,    8,    3,    9,   15,   24,    3,   26,   29,   21,    3,   19,   27,    4,    3,    7,    2,   15,    5,   18,   14,   27,    6,   25,   27,    8,   24,    4,   12,   11,   19,    4,   22,   29,   10,   29,   10,   18,   13,   16,    1,   29,    4,    3,   20,   10,   19,   23,   14,   14,    1,    3,   20,   16,   10,   28,   16,    3,   13,    4,   26,   26,   12,   20,   15,   12,    8,    5,   17,   21,   12,   23,   21,   24,   27,   18,    1,   11,   11,   27,   25,    4,    7,   15,    5,    7,   26,   17,   17,    2,    6,    2,   19,   23,   10,   13,    6,   16,   22,    1,   20,   23,    5,    1,   25,   10,   14,   22,   28,    4,   22,   23,    4,   28,   22,   12,    4,   16,    5,    8,   25,    2,   22,    9,   12,   27,   22,    7,    7,   13,    4,   18,   10,   23,   21,    4,    4,    8,    1,    8,   15,   22,   18,   25,   28,   26,    3,    9,   28,   24,   21,   29,   10,   22,   11,    7,   16,    2,   21,   14,    8,   16,   12,   22,    7,   20,   25,   10],
			// release times:
			vec![1324,  449, 1941, 2398, 2922, 1915,  380, 2894, 1224, 2735, 2531, 2362,  689, 2607, 1856, 1136, 2105, 2140, 1313,  293, 2294, 2191, 1440,  183, 1343,   35, 2041,  480,  146, 2406, 1396, 2736, 1818, 1975,  105, 2251,  606, 2220, 1081, 2734,  519, 2052,   81, 1276, 1778,  325, 2556, 2459,  529, 1998, 2141,  886, 2659, 2864, 2943, 2574, 1800, 2191, 2500, 2474, 2305, 1984, 2654, 1824,  973, 2557, 2839, 1876, 1652, 1107,  589, 1327,  900,  242,  119, 2453,  699,  776, 2742, 1774, 2595, 2111, 1221,  349, 1725, 1864, 1285, 2831, 2461, 1581, 2880, 1325,  664, 1750, 2427,  470,  703, 2169, 1928, 2257, 2969, 2239,  252,  557,  139,  463, 1099, 2593, 2759, 2222, 2318, 2411,  115,  174, 2408, 2038, 2342, 1487, 1595,  790,  454, 1797, 2893,  653,  674,  642,  477, 2817, 1798, 2140, 2586, 1302,  599, 2004, 2227, 1868, 1143,  631, 1818, 2835,   48, 1084, 2183, 2097, 2028, 1015,  898,  142,  453, 2764, 2065, 2904, 1062,  138,  895,  124, 1988,  332, 1878,  560, 1121, 1992,  693,  544, 1168,  889, 2793, 1665,  380, 1824,  477,   69, 2688, 2863, 1991, 2295, 1702, 2534, 1816,  996, 1935,   48, 2886,  239,  632,  623, 2703, 2559, 1471, 2924,  629, 1258,  596, 1913, 1749, 1927, 1500, 2332,  824, 2924],
			// due times:
			vec![2843, 1751, 1729, 2279, 1523, 1534, 2497, 2193, 1617, 2090, 1996, 2029, 1728, 2885, 1777, 2237, 2251, 2193, 1960, 2115, 2088, 1998, 2886, 1985, 1826, 1572, 2946, 2644, 2229, 2271, 1933, 2504, 1964, 2027, 2900, 1531, 2703, 2159, 1750, 1780, 2855, 1607, 2635, 1899, 1904, 2415, 2574, 2457, 2982, 1608, 1849, 2306, 2413, 2231, 1560, 2623, 2200, 2959, 2625, 2215, 2302, 1726, 2871, 2131, 2843, 1605, 2148, 2020, 2821, 2664, 1707, 2709, 1820, 2587, 2158, 2461, 1973, 2722, 2946, 1840, 1625, 2765, 2525, 1506, 1903, 1613, 1877, 2741, 1619, 2253, 2238, 1666, 2975, 1532, 2103, 2203, 2938, 2438, 2740, 2313, 2226, 1784, 1570, 1540, 2351, 2786, 2272, 1562, 2166, 1536, 2719, 1731, 1984, 2005, 2648, 2726, 2575, 1810, 1558, 2596, 2596, 2622, 2375, 2962, 1559, 2124, 1784, 2796, 1940, 2426, 1906, 1725, 2389, 1793, 1517, 2600, 1586, 2411, 1947, 2168, 1826, 2876, 2893, 2576, 2797, 2817, 1814, 1774, 2335, 1847, 2454, 1795, 2096, 1507, 1827, 1936, 2902, 1611, 1549, 1664, 2775, 2013, 2999, 2109, 2184, 1556, 2324, 2197, 1745, 2041, 2278, 2102, 2106, 2964, 2124, 2463, 2845, 1509, 1793, 2566, 1803, 1667, 1731, 1663, 2298, 2850, 1779, 2441, 2272, 1745, 1606, 2256, 2932, 2665, 2432, 2944, 2078, 2008, 2527, 2920],
		)
	}

pub fn benchmark_carlier(c: &mut Criterion) {
	let (p, r, d) = example_200_a();
	c.bench_function("carlier", |b| b.iter(|| {
		let schedule = carlier(black_box(&p), black_box(&r), black_box(&d));
		assert_eq!(schedule.lateness(&d), 1415);
	}));
}
criterion_group!(benches, benchmark_carlier);


criterion_main!(benches);
