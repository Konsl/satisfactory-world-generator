import { readSatisfactoryVersion } from "./version-info";

const content = await Bun.file("extracted-resources.json").json();
const gameVersion = await readSatisfactoryVersion();

const resourceNodes = content.filter(e => e[0] === "BP_ResourceNode_C").map(e => ({
    name: e[1],
    location: e[2],
    resource: e[3],
    purity: e[4],
}));
const geysers = content.filter(e => e[0] === "BP_ResourceNodeGeyser_C").map(e => ({
    name: e[1],
    location: e[2],
    purity: e[3],
}));
const frackingCores = content.filter(e => e[0] === "BP_FrackingCore_C").map(e => ({
    name: e[1],
    location: e[2],
    resource: null,
    satellites: []
}));

content.filter(e => e[0] === "BP_FrackingSatellite_C").forEach(e => {
    const core = frackingCores.find(c => c.name === e[5]);
    core.resource ??= e[3];
    if (core.resource !== e[3]) throw "idk";

    core.satellites.push({
        name: e[1],
        location: e[2],
        purity: e[4]
    });
});

const world = { gameVersion, resourceNodes, geysers, frackingCores };
await Bun.write("../src/default-world.json", JSON.stringify(world));
