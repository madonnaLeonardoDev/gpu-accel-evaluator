pub fn formulas<'a>() -> [&'a str; 28] {
[
    // --- Primitives ---
    // Sphere
    "sqrt(x^2 + y^2 + z^2) - 1",
    // Ellipsoid (approximate distance)
    "sqrt((x/1.5)^2 + y^2 + (z/0.8)^2) - 1",
    // Rounded box
    "smoothmax(smoothmax(abs(x) - 0.8, abs(y) - 0.8, 0.1), abs(z) - 0.8, 0.1)",
    // Torus
    "sqrt((sqrt(x^2 + z^2) - 1)^2 + y^2) - 0.35",
    // Capped cylinder
    "smoothmax(sqrt(x^2 + z^2) - 0.7, abs(y) - 1, 0.05)",
    // Capsule
    "sqrt(x^2 + (y - clamp(y, -0.5, 0.5))^2 + z^2) - 0.4",
    // Octahedron
    "(abs(x) + abs(y) + abs(z) - 1) * 0.57735",
    // Superquadric (blobby cube)
    "(x^4 + y^4 + z^4)^0.25 - 1",

    // --- Smooth booleans ---
    // Two spheres, smooth union
    "smoothmin(sqrt((x - 0.7)^2 + y^2 + z^2) - 0.7, sqrt((x + 0.7)^2 + y^2 + z^2) - 0.7, 0.6)",
    // Three-sphere blob
    "smoothmin(smoothmin(sqrt((x - 0.8)^2 + y^2 + z^2) - 0.55, sqrt((x + 0.8)^2 + y^2 + z^2) - 0.55, 0.5), sqrt(x^2 + (y - 0.9)^2 + z^2) - 0.55, 0.5)",
    // Box minus sphere (smooth subtraction)
    "smoothmax(smoothmax(smoothmax(abs(x) - 0.9, abs(y) - 0.9, 0.02), abs(z) - 0.9, 0.02), -(sqrt(x^2 + y^2 + z^2) - 1.15), 0.05)",
    // Sphere intersect box
    "smoothmax(sqrt(x^2 + y^2 + z^2) - 1.2, smoothmax(smoothmax(abs(x) - 0.9, abs(y) - 0.9, 0.02), abs(z) - 0.9, 0.02), 0.1)",
    // Torus smoothly fused with a sphere
    "smoothmin(sqrt((sqrt(x^2 + z^2) - 1)^2 + y^2) - 0.25, sqrt(x^2 + y^2 + z^2) - 0.6, 0.4)",

    // --- Mix / morph ---
    // Halfway between sphere and box
    "mix(sqrt(x^2 + y^2 + z^2) - 1, smoothmax(smoothmax(abs(x) - 0.8, abs(y) - 0.8, 0.02), abs(z) - 0.8, 0.02), 0.5)",
    // Sphere morphing into a torus along x
    "mix(sqrt(x^2 + y^2 + z^2) - 1, sqrt((sqrt(x^2 + z^2) - 1)^2 + y^2) - 0.35, clamp(x * 0.5 + 0.5, 0, 1))",

    // --- Domain repetition (mod) ---
    // Infinite grid of spheres
    "sqrt((mod(x + 1, 2) - 1)^2 + (mod(y + 1, 2) - 1)^2 + (mod(z + 1, 2) - 1)^2) - 0.4",
    // Infinite columns repeating in x and z
    "smoothmax(sqrt((mod(x + 1.5, 3) - 1.5)^2 + (mod(z + 1.5, 3) - 1.5)^2) - 0.5, abs(y) - 2, 0.05)",

    // --- Trig displacement and patterns ---
    // Bumpy sphere
    "sqrt(x^2 + y^2 + z^2) - 1 + sin(x * 8) * sin(y * 8) * sin(z * 8) * 0.05",
    // Ripples across a plane
    "y + 0.8 + sin(x * 3) * cos(z * 3) * 0.2",
    // Gyroid lattice clipped to a sphere
    "smoothmax(abs(sin(x * 3) * cos(y * 3) + sin(y * 3) * cos(z * 3) + sin(z * 3) * cos(x * 3)) / 3 - 0.05, sqrt(x^2 + y^2 + z^2) - 1.6, 0.1)",
    // Schwarz-P surface clipped to a sphere
    "smoothmax(abs(cos(x * 3) + cos(y * 3) + cos(z * 3)) / 3 - 0.06, sqrt(x^2 + y^2 + z^2) - 1.6, 0.1)",
    // Hollow shell with the top sliced off
    "smoothmax(abs(sqrt(x^2 + y^2 + z^2) - 1) - 0.05, y - 0.3, 0.02)",
    // Wavy radius (flower-ish blob)
    "sqrt(x^2 + y^2 + z^2) - 1 + 0.12 * sin(x * 5) * sin(y * 5) * sin(z * 5)",

    // --- Log / divide / tan / clamp ---
    // Log blob
    "ln(1 + x^2 + y^2 + z^2) - 0.7",
    // log2 blob (softer falloff)
    "log2(1 + x^2 + y^2 + z^2) - 1",
    // Metaballs via division (not a true distance field; the step scale handles it)
    "1.5 - (1 / ((x - 0.6)^2 + y^2 + z^2 + 0.01) + 1 / ((x + 0.6)^2 + y^2 + z^2 + 0.01)) * 0.25",
    // Clamped tan warp
    "sqrt(x^2 + y^2 + z^2) - 1 + 0.05 * clamp(tan(x * 3), -1, 1)",
    // Pillow
    "smoothmax(sqrt(x^2 + y^2 + z^2) - 1.3, abs(z) - 0.5 - 0.2 * cos(x * 2) * cos(y * 2), 0.1)",
]
}

