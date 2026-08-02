export interface ItemIconPixels {
  readonly width: number;
  readonly height: number;
  readonly rgba: Uint8ClampedArray;
}

export function decodeItemIconTga(bytes: ArrayBuffer): ItemIconPixels {
  const source = new Uint8Array(bytes);
  if (source.byteLength < 18) throw new Error("Item icon TGA is shorter than its header");
  const view = new DataView(bytes);
  const idLength = source[0];
  const colorMapType = source[1];
  const imageType = source[2];
  const width = view.getUint16(12, true);
  const height = view.getUint16(14, true);
  const pixelDepth = source[16];
  const descriptor = source[17];
  if (
    colorMapType !== 0
    || imageType !== 2
    || width === 0
    || height === 0
    || pixelDepth !== 32
    || (descriptor & 0x0f) !== 8
  ) {
    throw new Error("Item icon preview requires an uncompressed 32-bit RGBA TGA");
  }
  const pixelOffset = 18 + idLength;
  const pixelLength = width * height * 4;
  if (pixelOffset + pixelLength > source.byteLength) {
    throw new Error("Item icon TGA pixel payload is truncated");
  }
  const rightToLeft = (descriptor & 0x10) !== 0;
  const topToBottom = (descriptor & 0x20) !== 0;
  const rgba = new Uint8ClampedArray(pixelLength);
  for (let sourceY = 0; sourceY < height; sourceY += 1) {
    for (let sourceX = 0; sourceX < width; sourceX += 1) {
      const targetX = rightToLeft ? width - sourceX - 1 : sourceX;
      const targetY = topToBottom ? sourceY : height - sourceY - 1;
      const sourceOffset = pixelOffset + (sourceY * width + sourceX) * 4;
      const targetOffset = (targetY * width + targetX) * 4;
      rgba[targetOffset] = source[sourceOffset + 2];
      rgba[targetOffset + 1] = source[sourceOffset + 1];
      rgba[targetOffset + 2] = source[sourceOffset];
      rgba[targetOffset + 3] = source[sourceOffset + 3];
    }
  }
  return { width, height, rgba };
}

export function compositeItemIconLayers(
  layers: readonly ItemIconPixels[],
): ItemIconPixels {
  const first = layers[0];
  if (!first) throw new Error("At least one item icon layer is required");
  if (layers.some((layer) => layer.width !== first.width || layer.height !== first.height)) {
    throw new Error("Every item icon layer must use the same canvas");
  }
  const rgba = new Uint8ClampedArray(first.rgba.length);
  for (const layer of layers) {
    for (let offset = 0; offset < rgba.length; offset += 4) {
      const sourceAlpha = layer.rgba[offset + 3] / 255;
      const destinationAlpha = rgba[offset + 3] / 255;
      const outputAlpha = sourceAlpha + destinationAlpha * (1 - sourceAlpha);
      if (outputAlpha === 0) continue;
      for (let channel = 0; channel < 3; channel += 1) {
        rgba[offset + channel] = Math.round((
          layer.rgba[offset + channel] * sourceAlpha
          + rgba[offset + channel] * destinationAlpha * (1 - sourceAlpha)
        ) / outputAlpha);
      }
      rgba[offset + 3] = Math.round(outputAlpha * 255);
    }
  }
  return { width: first.width, height: first.height, rgba };
}
