const NWN_DDS_HEADER_BYTES = 20;
const MAX_TEXTURE_DIMENSION = 16_384;
const MAX_MIP_LEVELS = 32;

export interface NwnDdsMipmapV1 {
  readonly data: Uint8Array;
  readonly width: number;
  readonly height: number;
}

export interface NwnDdsV1 {
  readonly width: number;
  readonly height: number;
  readonly format: "DXT1" | "DXT5";
  readonly alphaMean: number;
  readonly mipmaps: readonly NwnDdsMipmapV1[];
}

function mipByteLength(width: number, height: number, blockBytes: number) {
  return Math.max(1, Math.ceil(width / 4)) * Math.max(1, Math.ceil(height / 4)) * blockBytes;
}

export function parseNwnDdsV1(buffer: ArrayBuffer): NwnDdsV1 {
  if (buffer.byteLength <= NWN_DDS_HEADER_BYTES) throw new Error("NWN DDS requires a 20-byte header and mip payload");
  const view = new DataView(buffer);
  const width = view.getUint32(0, true);
  const height = view.getUint32(4, true);
  const channels = view.getUint32(8, true);
  const linearSize = view.getUint32(12, true);
  const alphaMean = view.getFloat32(16, true);
  if (width === 0 || height === 0 || width > MAX_TEXTURE_DIMENSION || height > MAX_TEXTURE_DIMENSION) {
    throw new Error(`NWN DDS dimensions ${width}x${height} are invalid`);
  }
  if (channels !== 3 && channels !== 4) throw new Error(`NWN DDS channels ${channels} are unsupported`);
  if (!Number.isFinite(alphaMean)) throw new Error("NWN DDS alpha mean is invalid");
  const blockBytes = channels === 3 ? 8 : 16;
  const expectedLinearSize = mipByteLength(width, height, blockBytes);
  if (linearSize !== expectedLinearSize) {
    throw new Error(`NWN DDS top-level linear size ${linearSize} differs from ${expectedLinearSize}`);
  }

  const bytes = new Uint8Array(buffer);
  const mipmaps: NwnDdsMipmapV1[] = [];
  let offset = NWN_DDS_HEADER_BYTES;
  let mipWidth = width;
  let mipHeight = height;
  while (offset < bytes.byteLength && mipmaps.length < MAX_MIP_LEVELS) {
    const byteLength = mipByteLength(mipWidth, mipHeight, blockBytes);
    const end = offset + byteLength;
    if (end > bytes.byteLength) throw new Error("NWN DDS has a partial mip payload");
    mipmaps.push({ data: bytes.slice(offset, end), width: mipWidth, height: mipHeight });
    offset = end;
    mipWidth = Math.max(1, Math.floor(mipWidth / 2));
    mipHeight = Math.max(1, Math.floor(mipHeight / 2));
  }
  if (offset !== bytes.byteLength || mipmaps.length === MAX_MIP_LEVELS) {
    throw new Error("NWN DDS mip payload does not form a complete bounded chain");
  }
  return { width, height, format: channels === 3 ? "DXT1" : "DXT5", alphaMean, mipmaps };
}
