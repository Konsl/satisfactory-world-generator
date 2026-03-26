import { $ } from "bun";

class Reader {
    constructor(/** @type Buffer */ buffer) {
        /** @type Buffer */
        this.buffer = buffer;
        /** @type number */
        this._offset = 0;
        /** @type TextDecoder */
        this.decoder = new TextDecoder("utf-16");
    }

    /**
     * @function
     * @template T
     * @param {number} end
     * @param {() => T} func
     * @returns {A[]}
     */
    until(end, func) {
        const arr = [];
        while (this._offset < end) {
            arr.push(func());
        }
        return arr;
    }

    align(/** @type number */ n) {
        const offset = this._offset % n;
        const change = (n - offset) % n;
        this._offset += change;
    }

    skip(/** @type number */ n) {
        this._offset += n;
    }

    skipTo(/** @type number */ n) {
        this._offset = n;
    }

    offset() {
        return this._offset;
    }

    u16() {
        const val = this.buffer.readUInt16LE(this._offset);
        this._offset += 2;
        return val;
    }

    u32() {
        const val = this.buffer.readUInt32LE(this._offset);
        this._offset += 4;
        return val;
    }

    string(/** @type number | undefined */ n) {
        if (!n) {
            n = 0;
            while (this.buffer.readUInt16LE(this._offset + 2 * n) != 0)
                n++;
            n++;
        }

        const buffer = this.buffer.subarray(this._offset, this._offset + 2 * n);
        this._offset += 2 * n;
        return this.decoder.decode(buffer).replace(/\0$/, "");
    }
}

const readVar = (/** @type Reader */ reader) => {
    const end = reader.offset() + reader.u16();
    reader.skip(4); // value length, type
    const key = reader.string();
    reader.align(4);
    const value = reader.until(end, () => reader.u32());
    reader.align(4);

    return [key, value];
};

const readString = (/** @type Reader */ reader) => {
    const end = reader.offset() + reader.u16();
    reader.skip(4); // value length, type
    const key = reader.string();
    reader.align(4);
    const value = reader.string((end - reader.offset()) / 2);
    reader.align(4);

    return [key, value];
};

const readStringTable = (/** @type Reader */ reader) => {
    const end = reader.offset() + reader.u16();
    reader.skip(4); // value length, type
    const key = reader.string(8);
    reader.align(4);

    const children = Object.fromEntries(
        reader.until(end, () => readString(reader))
    );
    reader.align(4);

    return [key, children];
};

const readChildEntry = (/** @type Reader */ reader) => {
    const end = reader.offset() + reader.u16();
    reader.skip(4); // value length, type
    const type = reader.string();
    reader.align(4);

    let readFunc = () => { throw "unsupported entry"; };
    if (type === "StringFileInfo")
        readFunc = readStringTable;
    else if (type === "VarFileInfo")
        readFunc = readVar;

    const children = Object.fromEntries(
        reader.until(end, () => readFunc(reader))
    );
    reader.align(4);

    return { type, children };
};

const readFixedFileInfo = (/** @type Reader */ reader) => {
    const signature = reader.u32();
    if (signature != 0xFEEF04BD) throw `invalid signature ${signature.toString(16)}`;
    reader.skip(4); // struc version

    const fileVersionH = reader.u32();
    const fileVersionL = reader.u32();
    const fileVersion = [
        fileVersionH >> 16, fileVersionH & 0xFFFF,
        fileVersionL >> 16, fileVersionL & 0xFF
    ];
    const productVersionH = reader.u32();
    const productVersionL = reader.u32();
    const productVersion = [
        productVersionH >> 16, productVersionH & 0xFFFF,
        productVersionL >> 16, productVersionL & 0xFF
    ];

    // ...

    return { fileVersion, productVersion };
};

const readVersionInfo = (/** @type Reader */ reader) => {
    const end = reader.offset() + reader.u16();
    const valueLength = reader.u16();
    reader.skip(2); // type
    const key = reader.string(16);
    if (key != "VS_VERSION_INFO") throw "invalid version info key";
    reader.align(4);

    let fixedInfo;
    if (valueLength > 0) {
        const valueEnd = reader.offset() + valueLength;
        fixedInfo = readFixedFileInfo(reader);
        reader.skipTo(valueEnd);
    }
    reader.align(4);

    const entries = reader.until(end, () => readChildEntry(reader));
    return { fixedInfo, entries };
};

/** @returns {Promise<string>} */
export async function readSatisfactoryVersion() {
    const SATISFACTORY_PATH = ".../steam/steamapps/common/Satisfactory";
    const dllPath = `${SATISFACTORY_PATH}/FactoryGame/Binaries/Win64/FactoryGameSteam-FactoryGame-Win64-Shipping.dll`;

    const { stdout } = await $`wrestool --extract --raw --type=version ${dllPath}`.quiet();
    const reader = new Reader(stdout);

    const versionInfo = readVersionInfo(reader);
    return Object.values(
        versionInfo.entries
            .find(c => c.type === "StringFileInfo")
            .children
    )[0]["FileVersion"];
}

