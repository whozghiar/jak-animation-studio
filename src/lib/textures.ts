// The game's textures as the decompiler writes them (decompiler_out/<game>/textures/<page>/*.png),
// with the PS2's alpha: 128 is opaque.

import { readModel, type EyeFrame } from "$lib/rpc";

/** A texture file's pixels. */
export async function pixels(path: string) {
  const image = await createImageBitmap(new Blob([await readModel(path)]), {
    premultiplyAlpha: "none",
    colorSpaceConversion: "none",
  });
  const c = new OffscreenCanvas(image.width, image.height).getContext("2d")!;
  c.drawImage(image, 0, 0);
  return c.getImageData(0, 0, image.width, image.height);
}

/** The pixels drawn as the PS2 blends them: alpha 128 covers. */
function opaque(image: ImageData) {
  const data = new Uint8ClampedArray(image.data);
  for (let i = 3; i < data.length; i += 4) data[i] *= 2;
  const c = new OffscreenCanvas(image.width, image.height);
  c.getContext("2d")!.putImageData(
    new ImageData(data, image.width, image.height),
    0,
    0,
  );
  return c;
}

/** The pixels drawn without blending: the texture's colors over everything. */
function solid(image: ImageData) {
  const data = new Uint8ClampedArray(image.data);
  for (let i = 3; i < data.length; i += 4) data[i] = 255;
  const c = new OffscreenCanvas(image.width, image.height);
  c.getContext("2d")!.putImageData(
    new ImageData(data, image.width, image.height),
    0,
    0,
  );
  return c;
}

/** An eye at one frame of the animation's eye data (`EyeFrame`), as the game draws it (eye.gc
 * render-eyes, OpenGOAL's EyeRenderer): cleared with the iris texture's corner texel, then the iris
 * (unblended) and the pupil (blended by its alpha), squares `size` of the eye wide centered at
 * (½ + x, ½ + y) of it, then the lid, unblended over the eye's width from `lid - 1` to
 * `lid - 1 + lidScale` of its height (an open lid, below 0, at `1 + lid`; the game blinks it now
 * and then). The right eye takes the left one's lid size (eye.gc does). */
export function eye(
  iris: ImageData,
  pupil: ImageData,
  lid: ImageData | null,
  [x, y, l, irisScale, pupilScale]: EyeFrame,
  lidScale: number,
  size = 64,
) {
  const c = new OffscreenCanvas(size, size).getContext("2d")!;
  const [r, g, b] = iris.data;
  c.fillStyle = `rgb(${r},${g},${b})`;
  c.fillRect(0, 0, size, size);
  const cx = size * (0.5 + x);
  const cy = size * (0.5 + y);
  for (const [image, scale] of [
    [solid(iris), irisScale],
    [opaque(pupil), pupilScale],
  ] as const) {
    const w = size * scale;
    if (w > 0) c.drawImage(image, cx - w / 2, cy - w / 2, w, w);
  }
  const top = l < 0 ? 1 + l : l;
  if (lid && lidScale > 0)
    c.drawImage(solid(lid), 0, size * (top - 1), size, size * lidScale);
  return c.getImageData(0, 0, size, size);
}

/** Dark Jak's skin: `norm` turned into `dark` by `amount`, as the game's texture animation does
 * (texture-anim-tables.gc); the alpha stays the model's. */
export function blend(norm: ImageData, dark: ImageData, amount: number) {
  const out = new Uint8ClampedArray(norm.data);
  for (let i = 0; i < out.length; i++)
    if (i % 4 !== 3) out[i] += (dark.data[i] - norm.data[i]) * amount;
  return new ImageData(out, norm.width, norm.height);
}
