#r "nuget: CUE4Parse, 1.2.2.21"
#r "nuget: CUE4Parse-Conversion, 1.2.2.21"

using System.Diagnostics;
using CUE4Parse_Conversion.Textures;
using CUE4Parse.Compression;
using CUE4Parse.FileProvider;
using CUE4Parse.MappingsProvider;
using CUE4Parse.UE4.Assets.Exports.Texture;
using CUE4Parse.UE4.Objects.Core.Math;
using CUE4Parse.UE4.Objects.Engine;
using CUE4Parse.UE4.Objects.UObject;
using CUE4Parse.UE4.Versions;
using SkiaSharp;

string satisfactoryPath = null;
foreach (var steamPath in new[] {
    $"{Environment.GetFolderPath(Environment.SpecialFolder.UserProfile)}/.steam/root/",
    "C:/Program Files (x86)/steam/",
}) {
    var gamePath = $"{steamPath}steamapps/common/Satisfactory/";
    
    if (Path.Exists(gamePath))
        satisfactoryPath = gamePath;
}

if (satisfactoryPath is null) {
    Console.Error.WriteLine("satisfactory installation not found. exiting");
    return;
}

var packageDir = $"{satisfactoryPath}FactoryGame/Content/Paks/";
var mappingsFile = $"{satisfactoryPath}CommunityResources/FactoryGame.usmap";
var mainDllFile = $"{satisfactoryPath}FactoryGame/Binaries/Win64/FactoryGameSteam-FactoryGame-Win64-Shipping.dll";

var version = FileVersionInfo.GetVersionInfo(mainDllFile);
var ueVersion = $"{version.FileMajorPart}_{version.FileMinorPart}";
var gameVersion = version.FileVersion;

if (string.IsNullOrWhiteSpace(gameVersion) || ueVersion == "0_0")
{
    Console.Error.WriteLine("FileVersionInfo doesn't work. trying again using mono csharp cli...");
    var script = @"
var version = System.Diagnostics.FileVersionInfo.GetVersionInfo(Args[0]);
Console.WriteLine($""{version.FileMajorPart}_{version.FileMinorPart}"");
Console.Write(version.FileVersion);
";
    var process = Process.Start(new ProcessStartInfo()
    {
        FileName = "csharp",
        ArgumentList = { "-s", "/dev/stdin", mainDllFile },
        RedirectStandardInput = true,
        RedirectStandardOutput = true,
        UseShellExecute = false,
    });
    if (process is null)
    {
        Console.Error.WriteLine("could not start csharp process. exiting");
        return;
    }

    process.StandardInput.WriteLine(script);
    process.StandardInput.Close();

    ueVersion = process.StandardOutput.ReadLine();
    gameVersion = process.StandardOutput.ReadToEnd();
}

var ueVersionString = $"GAME_UE{ueVersion}";
EGame game;
try
{
    game = Enum.Parse<EGame>(ueVersionString);
}
catch (Exception)
{
    Console.Error.WriteLine($"unsupported ue version {ueVersionString}. exiting");
    return;
}

foreach (var libName in new[] {
    "liboo2corelinux64.so.9",
    "oodle-data-shared.dll",
    "oo2core_9_win64.dll",
}) {
    var oodlePath = Path.Combine(".", libName);

    if (Path.Exists(oodlePath)) {
        OodleHelper.Initialize(oodlePath);
        break;
    }
}

var provider = new DefaultFileProvider(packageDir, SearchOption.AllDirectories, new VersionContainer(game),
    StringComparer.Ordinal);
try
{
    provider.MappingsContainer = new FileUsmapTypeMappingsProvider(mappingsFile);
}
catch (Exception)
{
    Console.Error.WriteLine("could not load mappings file. continuing without mappings...");
}

provider.Initialize();
provider.Mount();


const string levelPath = "FactoryGame/Content/FactoryGame/Map/GameLevel01/Persistent_Level.umap.PersistentLevel";
var level = provider.LoadPackageObject<ULevel>(levelPath);

var seenResources = new HashSet<UBlueprintGeneratedClass>();
// BP_ResourceNode_C, BP_FrackingSatellite_C, BP_FrackingCore_C, BP_ResourceNodeGeyser_C

using var writer = new StreamWriter("extracted-resources.json");
writer.WriteLine("[");
writer.WriteLine($"""["GameVersion", "{gameVersion}"],""");

foreach (var node in level.Actors.Select(a => a.Load()).Where(a => a is { ExportType: "BP_ResourceNode_C" }))
{
    if (node is null) continue;


    var name = node.Name;
    var location = node.Get<FPackageIndex>("mBoxComponent").Load().Get<FVector>("RelativeLocation");
    var resource = node.Get<FPackageIndex>("mResourceClass");
    seenResources.Add(resource.Load<UBlueprintGeneratedClass>());
    var purity = node.GetOrDefault<FName>("mPurity", "RP_Normal").ToString();

    writer.WriteLine(
        $"""["{node.ExportType}", "{name}", [{location.X}, {location.Y}, {location.Z}], "{resource.Name}", "{purity}"],""");
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
    var resource = node.Get<FPackageIndex>("mResourceClass");
    seenResources.Add(resource.Load<UBlueprintGeneratedClass>());
    var purity = node.GetOrDefault<FName>("mPurity", "RP_Normal").ToString();
    var core = node.Get<FPackageIndex>("mCore").Name;

    writer.WriteLine(
        $"""["{node.ExportType}", "{name}", [{location.X}, {location.Y}, {location.Z}], "{resource.Name}", "{purity}", "{core}"],""");
}

writer.WriteLine("[null]]");

const string geoGeneratorPath =
    "FactoryGame/Content/FactoryGame/Buildable/Factory/GeneratorGeoThermal/Desc_GeneratorGeoThermal.uasset.Desc_GeneratorGeoThermal_C";
var geoGenerator = provider.LoadPackageObject<UBlueprintGeneratedClass>(geoGeneratorPath).ClassDefaultObject.Load();
var geoGeneratorTexture = geoGenerator.Get<UTexture2D>("mSmallIcon");

var textures = new Dictionary<string, UTexture2D>();
textures[geoGenerator.Class.Name.ToString()] = geoGeneratorTexture;

foreach (var resourceClass in seenResources)
{
    var resource = resourceClass.ClassDefaultObject.Load();
    resource.TryGetValue(out UTexture2D icon, "mPersistentBigIcon", "mSmallIcon");
    textures[resourceClass.Name] = icon;
}

const string iconOutputDir = "textures";
Directory.CreateDirectory(iconOutputDir);

foreach (var (name, texture) in textures)
{
    var bitmap = texture.Decode();
    if (bitmap is null)
    {
        Console.Error.WriteLine($"could not load {name} texture");
        continue;
    }
    
    using var stream = File.Create(Path.Combine(iconOutputDir, $"{name}.png"));
    bitmap.ToSkBitmap().Encode(SKEncodedImageFormat.Png, 100).SaveTo(stream);
}
