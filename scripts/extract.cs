using System.Diagnostics;
using CUE4Parse.Compression;
using CUE4Parse.FileProvider;
using CUE4Parse.MappingsProvider;
using CUE4Parse.UE4.Assets.Exports;
using CUE4Parse.UE4.Assets.Exports.Actor;
using CUE4Parse.UE4.Assets.Exports.Component;
using CUE4Parse.UE4.Assets.Exports.Texture;
using CUE4Parse.UE4.Objects.Core.Math;
using CUE4Parse.UE4.Objects.Engine;
using CUE4Parse.UE4.Objects.UObject;
using CUE4Parse.UE4.Versions;
using Newtonsoft.Json;

const string directory = ".../steam/steamapps/common/Satisfactory/FactoryGame/Content/Paks/";
const string mapping = ".../steam/steamapps/common/Satisfactory/CommunityResources/FactoryGame.usmap";
const string levelPath = "FactoryGame/Content/FactoryGame/Map/GameLevel01/Persistent_Level.umap.PersistentLevel";

var oodlePath = Path.Combine(".", "liboo2corelinux64.so.9");
OodleHelper.Initialize(oodlePath);

var provider = new DefaultFileProvider(directory, SearchOption.AllDirectories, new VersionContainer(EGame.GAME_UE5_6),
    StringComparer.Ordinal);
// provider.MappingsContainer = new FileUsmapTypeMappingsProvider(mapping);
provider.Initialize();
provider.Mount();


// BP_ResourceNode_C, BP_FrackingSatellite_C, BP_FrackingCore_C, BP_ResourceNodeGeyser_C
var level = provider.LoadPackageObject<ULevel>(levelPath);

using var writer = new StreamWriter("extracted-resources.json");
writer.WriteLine("[");

foreach (var node in level.Actors.Select(a => a.Load()).Where(a => a is { ExportType: "BP_ResourceNode_C" }))
{
    if (node is null) continue;

    var name = node.Name;
    var location = node.Get<FPackageIndex>("mBoxComponent").Load().Get<FVector>("RelativeLocation");
    var resource = node.Get<FPackageIndex>("mResourceClass").Name;
    var purity = node.GetOrDefault<FName>("mPurity", "RP_Normal").ToString();

    writer.WriteLine(
        $"""["{node.ExportType}", "{name}", [{location.X}, {location.Y}, {location.Z}], "{resource}", "{purity}"],""");
}

foreach (var node in level.Actors.Select(a => a.Load()).Where(a => a is { ExportType: "BP_ResourceNodeGeyser_C" }))
{
    if (node is null) continue;

    var name = node.Name;
    var location = node.Get<FPackageIndex>("mBoxComponent").Load().Get<FVector>("RelativeLocation");
    var purity = node.GetOrDefault<FName>("mPurity", "RP_Normal").ToString();

    writer.WriteLine(
        $"""["{node.ExportType}", "{name}", [{location.X}, {location.Y}, {location.Z}], "{purity}"],""");
}

foreach (var node in level.Actors.Select(a => a.Load()).Where(a => a is { ExportType: "BP_FrackingCore_C" }))
{
    if (node is null) continue;

    var name = node.Name;
    var location = node.Get<FPackageIndex>("mBoxComponent").Load().Get<FVector>("RelativeLocation");

    writer.WriteLine(
        $"""["{node.ExportType}", "{name}", [{location.X}, {location.Y}, {location.Z}]],""");
}

foreach (var node in level.Actors.Select(a => a.Load()).Where(a => a is { ExportType: "BP_FrackingSatellite_C" }))
{
    if (node is null) continue;

    var name = node.Name;
    var location = node.Get<FPackageIndex>("mBoxComponent").Load().Get<FVector>("RelativeLocation");
    var resource = node.Get<FPackageIndex>("mResourceClass").Name;
    var purity = node.GetOrDefault<FName>("mPurity", "RP_Normal").ToString();
    var core = node.Get<FPackageIndex>("mCore").Name;

    writer.WriteLine(
        $"""["{node.ExportType}", "{name}", [{location.X}, {location.Y}, {location.Z}], "{resource}", "{purity}", "{core}"],""");
}

writer.WriteLine("]");

