// The game's textures as the decompiler writes them (decompiler_out/<game>/textures/<page>/*.png),
// with the PS2's alpha: 128 is opaque.

import { readModel } from "$lib/rpc";

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

/** An eye as the game draws it (eye.gc, OpenGOAL's EyeRenderer): the iris texture's corner color,
 * then the iris and the pupil centered, each `scale` of the eye wide. Without the eye animation
 * (not extracted), the eye looks ahead with its lid open: eye.gc's defaults. */
export function eye(
  iris: ImageData,
  pupil: ImageData,
  irisScale: number,
  pupilScale: number,
  size = 64,
) {
  const c = new OffscreenCanvas(size, size).getContext("2d")!;
  const [r, g, b] = iris.data;
  c.fillStyle = `rgb(${r},${g},${b})`;
  c.fillRect(0, 0, size, size);
  for (const [image, scale] of [
    [iris, irisScale],
    [pupil, pupilScale],
  ] as const) {
    const w = size * scale;
    c.drawImage(opaque(image), (size - w) / 2, (size - w) / 2, w, w);
  }
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
