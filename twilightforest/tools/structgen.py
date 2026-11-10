"""Translate the decompiled Twilight Forest structure classes into Rust drafts under src/world/structures.

Every StructureComponent subclass becomes a `Kind` of one `Piece` struct whose fields are the union of all class
fields, each in a Cell so pieces stay shared through `Rc` while they build. A class becomes a module of free functions
taking `this: &Piece`; methods that subclasses override get a dispatcher on `Piece` that matches on the kind. TFMaze
becomes its own struct the same way. StructureComponent and StructureTFComponent helpers are hand written in
component.rs under the names in HELPERS.

Needs javalang (pip install javalang==0.13.0 into a scratch venv):

    venv/bin/python3 -I tools/structgen.py reference/src/twilightforest/structures src/world/structures [--only Class,...]
"""

import re
import sys
from pathlib import Path

import javalang
from javalang import tree as T

ROOT = "StructureComponent"
BASES = {"StructureComponent", "StructureTFComponent"}

# StructureComponent and StructureTFComponent methods with their component.rs names and return types
HELPERS = {
    "func_151550_a": ("placeBlock", None), "func_151556_a": ("fillMeta", None), "func_151549_a": ("fill", None),
    "func_74882_a": ("fillSelected", None), "func_151551_a": ("fillRandomly", None),
    "func_151552_a": ("placeRandomly", None), "func_74878_a": ("fillAir", None), "func_151548_a": ("getBlock", "Block"),
    "func_151554_b": ("fillDown", None), "func_74871_b": ("clearUp", None), "func_74865_a": ("xAt", "int"),
    "func_74862_a": ("yAt", "int"), "func_74873_b": ("zAt", "int"), "func_74874_b": ("bounds", "StructureBoundingBox"),
    "func_74877_c": ("depth", "int"), "func_74860_a": ("liquidIn", "boolean"),
    "placeSpawnerAtCurrentPosition": ("placeSpawner", None), "placeSpawnerRotated": ("placeSpawnerRotated", None),
    "placeTreasureAtCurrentPosition": ("placeTreasure", None), "placeTreasureRotated": ("placeTreasureRotated", None),
    "placeSignAtCurrentPosition": ("placeSign", None), "offsetTowerCoords": ("offsetTower", "int[]"),
    "offsetTowerCCoords": ("offsetTowerAt", "ChunkCoordinates"), "getOffsetAsIfRotated": ("offsetTurned", "int[]"),
    "getXWithOffsetAsIfRotated": ("xTurned", "int"), "getZWithOffsetAsIfRotated": ("zTurned", "int"),
    "getCoordBaseMode": ("mode", "int"), "setCoordBaseMode": ("setMode", None),
    "placeBlockRotated": ("placeTurned", None), "getBlockIDRotated": ("getTurned", "Block"),
    "fillBlocksRotated": ("fillTurned", None), "randomlyFillBlocksRotated": ("fillRandomlyTurned", None),
    "fillToGroundRotated": ("fillDownTurned", None), "fillAirRotated": ("fillAirTurned", None),
    "getStairMeta": ("stairMeta", "int"), "getLadderMeta": ("ladderMeta", "int"),
    "nullifySkyLightForBoundingBox": ("darken", None), "nullifySkyLightAtCurrentPosition": ("darkenAt", None),
    "nullifySkyLight": ("darkenWorld", None), "getAverageGroundLevel": ("groundLevel", "int"),
    "getSampledDirtLevel": ("dirtLevel", "int"), "randomlyPlaceBlock": ("placeRandomly", None),
    "isComponentProtected": ("protected", "boolean"),
}
STATIC_HELPERS = {
    "func_74883_a": ("intersecting", "StructureComponent"), "findIntersectingExcluding": ("intersectingExcept", "StructureComponent"),
    "getComponentToAddBoundingBox": ("boxFor", "StructureBoundingBox"),
    "getComponentToAddBoundingBox2": ("boxFor2", "StructureBoundingBox"), "getStrongholdStones": ("Stones::stronghold", "BlockSelector"),
}
VIRTUAL = {"func_74861_a": "build", "func_74875_a": "add"}

BOX_FIELDS = {"field_78897_a": "m_min[0]", "field_78895_b": "m_min[1]", "field_78896_c": "m_min[2]",
              "field_78893_d": "m_max[0]", "field_78894_e": "m_max[1]", "field_78892_f": "m_max[2]"}
COORD_FIELDS = {"field_71574_a": "[0]", "field_71572_b": "[1]", "field_71573_c": "[2]"}
BOX_METHODS = {"func_78890_b": ("contains", "boolean"), "func_78884_a": ("intersects", "boolean"),
               "func_78885_a": ("intersectsXZ", "boolean"), "func_78883_b": ("xSize", "int"),
               "func_78882_c": ("ySize", "int"), "func_78880_d": ("zSize", "int"), "func_78881_e": ("centerX", "int"),
               "func_78891_g": ("centerZ", "int"), "func_78886_a": ("shift", None), "func_78888_b": ("expand", None)}
BOX_MUTATING = {"shift", "expand"}
WORLD_METHODS = {"func_147465_d": ("setBlock", None), "func_147439_a": ("getBlock", "Block"),
                 "func_72805_g": ("meta", "int"), "func_147437_c": ("isAir", "boolean"),
                 "func_147468_f": ("clear", None), "func_72905_C": ("seed", "long"),
                 "func_72807_a": ("biome", "BiomeGenBase"), "func_72825_h": ("topSolid", "int"),
                 "func_72838_d": ("spawn", None), "func_147438_o": ("tile", "TileEntity")}
RANDOM_METHODS = {"nextInt": ("nextIntBounded", "int"), "nextFloat": ("nextFloat", "float"),
                  "nextBoolean": ("nextBoolean", "boolean"), "nextDouble": ("nextDouble", "double"),
                  "nextLong": ("nextLong", "long"), "nextGaussian": ("nextGaussian", "double"), "setSeed": ("setSeed", None)}
MATH = {"func_76126_a": ("sin", "float"), "func_76134_b": ("cos", "float"), "func_76129_c": ("sqrtF", "float"),
        "func_76141_d": ("floorF", "int"), "func_76128_c": ("floor", "int"), "func_76143_f": ("ceil", "int")}

BLOCKS = {
    "150350_a": "air", "150348_b": "stone", "150349_c": "grass", "150346_d": "dirt", "150347_e": "cobblestone",
    "150344_f": "planks", "150345_g": "sapling", "150357_h": "bedrock", "150358_i": "flowing_water", "150355_j": "water",
    "150356_k": "flowing_lava", "150353_l": "lava", "150354_m": "sand", "150351_n": "gravel", "150352_o": "gold_ore",
    "150366_p": "iron_ore", "150365_q": "coal_ore", "150364_r": "log", "150362_t": "leaves", "150359_w": "glass",
    "150369_x": "lapis_ore", "150368_y": "lapis_block", "150367_z": "dispenser", "150322_A": "sandstone",
    "150320_F": "sticky_piston", "150321_G": "web", "150329_H": "tallgrass", "150330_I": "deadbush", "150331_J": "piston",
    "150325_L": "wool", "150327_N": "yellow_flower", "150328_O": "red_flower", "150338_P": "brown_mushroom",
    "150337_Q": "red_mushroom", "150340_R": "gold_block", "150339_S": "iron_block", "150334_T": "double_stone_slab",
    "150333_U": "stone_slab", "150336_V": "brick_block", "150335_W": "tnt", "150342_X": "bookshelf",
    "150341_Y": "mossy_cobblestone", "150343_Z": "obsidian", "150478_aa": "torch", "150480_ab": "fire",
    "150474_ac": "mob_spawner", "150476_ad": "oak_stairs", "150486_ae": "chest", "150488_af": "redstone_wire",
    "150482_ag": "diamond_ore", "150484_ah": "diamond_block", "150462_ai": "crafting_table", "150458_ak": "farmland",
    "150460_al": "furnace", "150472_an": "standing_sign", "150468_ap": "ladder", "150442_at": "lever",
    "150452_aw": "wooden_pressure_plate", "150450_ax": "redstone_ore", "150439_ay": "lit_redstone_ore",
    "150430_aB": "stone_button", "150431_aC": "snow_layer", "150432_aD": "ice", "150433_aE": "snow", "150434_aF": "cactus",
    "150422_aJ": "fence", "150423_aK": "pumpkin", "150424_aL": "netherrack", "150425_aM": "soul_sand",
    "150426_aN": "glowstone", "150427_aO": "portal", "150428_aP": "lit_pumpkin", "150413_aR": "unpowered_repeater",
    "150418_aU": "monster_egg", "150417_aV": "stonebrick", "150420_aW": "brown_mushroom_block",
    "150419_aX": "red_mushroom_block", "150411_aY": "iron_bars", "150410_aZ": "glass_pane", "150395_bd": "vine",
    "150390_bg": "stone_brick_stairs", "150391_bh": "mycelium", "150386_bk": "nether_brick_fence",
    "150388_bm": "nether_wart", "150382_bo": "brewing_stand", "150383_bp": "cauldron", "150384_bq": "end_portal",
    "150378_br": "end_portal_frame", "150377_bs": "end_stone", "150379_bu": "redstone_lamp", "150374_bv": "lit_redstone_lamp",
    "150373_bw": "double_wooden_slab", "150376_bx": "wooden_slab", "150412_bA": "emerald_ore",
    "150479_bC": "tripwire_hook", "150473_bD": "tripwire", "150475_bE": "emerald_block", "150485_bF": "spruce_stairs",
    "150487_bG": "birch_stairs", "150463_bK": "cobblestone_wall", "150457_bL": "flower_pot", "150467_bQ": "anvil",
    "150447_bR": "trapped_chest", "150451_bX": "redstone_block", "150449_bY": "quartz_ore", "150371_ca": "quartz_block",
    "150370_cb": "quartz_stairs", "150406_ce": "stained_hardened_clay", "150399_cn": "stained_glass",
    "150403_cj": "packed_ice", "150429_aA": "redstone_torch", "150415_aT": "trapdoor", "150389_bf": "brick_stairs",
    "150385_bj": "nether_brick", "150387_bl": "nether_brick_stairs", "150381_bn": "enchanting_table",
    "150372_bz": "sandstone_stairs", "150446_ar": "stone_stairs", "150466_ao": "wooden_door", "150454_av": "iron_door",
    "150456_au": "stone_pressure_plate", "150396_be": "fence_gate", "150440_ba": "melon_block", "150465_bP": "skull",
    "150361_u": "sponge", "150392_bi": "waterlily", "150438_bZ": "hopper", "150477_bB": "ender_chest",
    "150437_az": "unlit_redstone_torch", "150393_bb": "melon_stem", "150394_bc": "pumpkin_stem",
    "150398_cm": "double_plant", "150363_s": "log2", "150361_v": "leaves2", "150405_ch": "hardened_clay",
    "150404_cg": "carpet", "150407_cf": "hay_block", "150402_ci": "coal_block", "150400_ck": "acacia_stairs",
}

TREASURE = {"hill1": "Hill1", "hill2": "Hill2", "hill3": "Hill3", "hedgemaze": "HedgeMaze",
            "labyrinth_room": "LabyrinthRoom", "labyrinth_deadend": "LabyrinthDeadEnd", "tower_room": "TowerRoom",
            "tower_library": "TowerLibrary", "basement": "Basement", "labyrinth_vault": "LabyrinthVault",
            "darktower_cache": "DarkTowerCache", "darktower_key": "DarkTowerKey", "darktower_boss": "DarkTowerBoss",
            "tree_cache": "TreeCache", "stronghold_cache": "StrongholdCache", "stronghold_room": "StrongholdRoom",
            "stronghold_boss": "StrongholdBoss", "aurora_cache": "AuroraCache", "aurora_room": "AuroraRoom",
            "aurora_boss": "AuroraBoss", "troll_garden": "TrollGarden", "troll_vault": "TrollVault"}

FEATURES = {"hill1": "SmallHill", "hill2": "MediumHill", "hill3": "LargeHill", "tfStronghold": "Stronghold"}
FEATURES.update({name: name[0].upper() + name[1:] for name in ("hedgeMaze", "nagaCourtyard", "lichTower", "iceTower",
                 "questIsland", "questGrove", "druidGrove", "floatingRuins", "hydraLair", "labyrinth", "darkTower",
                 "worldTree", "yetiCave", "trollCave", "finalCastle", "mushroomTower")})

PRIMITIVE = {"int": "i32", "boolean": "bool", "float": "f32", "double": "f64", "byte": "i32", "short": "i32",
             "long": "i64", "char": "i32"}
WIDTH = {"int": 0, "byte": 0, "short": 0, "char": 0, "long": 1, "float": 2, "double": 3}
NUMERIC_RUST = {"int": "i32", "byte": "i32", "short": "i32", "char": "i32", "long": "i64", "float": "f32", "double": "f64"}
STONES = {"BlockSelector", "StructureTFStrongholdStones", "StructureTFMazeStones", "StructureTFTowerWoods",
          "StructureTFAuroraBricks", "StructureTFCastleBlocks", "StructureTFKnightStones"}
KEYWORDS = {"type", "move", "ref", "fn", "match", "loop", "impl", "mod", "box", "use", "crate", "self", "where", "in"}


def plainName(klass, name, arity):
    arities = sorted({len(m.parameters) for m in klass.methods if m.name == name})
    return camel(name) if len(arities) <= 1 or arity == arities[0] else f"{camel(name)}{arity}"


def snake(name):
    name = re.sub(r"^(ComponentTF|StructureTF|StructureDecorator)", "", name)
    return re.sub(r"(?<=[a-z0-9])(?=[A-Z])", "_", name).lower()


def camel(name):
    return name[0].lower() + name[1:] if name else name


def kindName(name):
    return name.replace("ComponentTF", "").replace("StructureTF", "")


def typeName(node):
    if node is None:
        return "void"
    name = node.name
    if isinstance(node, T.ReferenceType) and node.sub_type is not None:
        name = node.sub_type.name
    dims = len(node.dimensions or [])
    if isinstance(node, T.ReferenceType) and name in ("List", "ArrayList", "LinkedList"):
        inner = node.arguments[0].type if node.arguments and node.arguments[0].type is not None else None
        return (inner.name if inner is not None else "StructureComponent") + "[]" + "[]" * dims
    return name + "[]" * dims


class Klass:
    def __init__(self, path, decl, package):
        self.path, self.decl, self.package = path, decl, package
        self.name = decl.name
        self.parent = decl.extends.name if getattr(decl, "extends", None) is not None else None
        self.module = snake(self.name)
        self.abstract = "abstract" in decl.modifiers
        self.fields = {}
        self.renamed = {}
        self.statics = {}
        self.initializers = []
        for field in decl.fields:
            jtype = typeName(field.type)
            for declarator in field.declarators:
                if "static" in field.modifiers:
                    self.statics[declarator.name] = (jtype, declarator.initializer)
                else:
                    self.fields[declarator.name] = jtype
                    if declarator.initializer is not None:
                        self.initializers.append((declarator.name, declarator.initializer))
        self.methods = [m for m in decl.methods if m.name not in ("getDoorsAsIntArray", "readOpeningsFromArray") and not m.name.startswith("func_143") and not any(typeName(p.type) == "NBTTagCompound" for p in m.parameters)]
        self.constructors = [c for c in decl.constructors if c.parameters]


class Program:
    def __init__(self, source):
        self.classes = {}
        for path in sorted(Path(source).rglob("*.java")):
            unit = javalang.parse.parse(path.read_text(encoding="utf-8"))
            package = path.parent.name if path.parent != Path(source) else ""
            for decl in unit.types:
                if isinstance(decl, T.ClassDeclaration):
                    self.classes[decl.name] = Klass(path, decl, package)
        self.components = [k for k in self.classes.values() if self.isComponent(k.name) and k.name not in BASES]
        self.componentNames = {k.name for k in self.components}
        self.collectFields()
        self.collectMethods()

    def isComponent(self, name):
        while name in self.classes:
            name = self.classes[name].parent
        return name == ROOT

    def chain(self, name):
        out = []
        while name in self.classes and name not in BASES:
            out.append(name)
            name = self.classes[name].parent
        return out

    def descendants(self, name):
        return [k.name for k in self.components if name in self.chain(k.name)]

    def collectFields(self):
        self.fields = {}
        for klass in self.components:
            for name, jtype in klass.fields.items():
                key = name
                if key in self.fields and self.fields[key][0] != jtype:
                    key = f"{name}{kindName(klass.name)}"
                    klass.renamed[name] = key
                self.fields.setdefault(key, (jtype, klass.name))

    def fieldOf(self, klassName, name):
        owners = self.chain(klassName) if klassName in self.componentNames else [klassName]
        for owner in owners:
            klass = self.classes[owner]
            if name in klass.fields:
                return klass.renamed.get(name, name), klass.fields[name]
        return None

    def collectMethods(self):
        self.impls = {}
        self.signature = {}
        for klass in self.components:
            for method in klass.methods:
                if "static" in method.modifiers:
                    continue
                key = (method.name, len(method.parameters))
                if method.body is not None:
                    self.impls.setdefault(key, []).append(klass.name)
                self.signature.setdefault(key, method)
        self.nullReturns = set()
        for klass in self.classes.values():
            for method in klass.methods:
                if any(isinstance(child, T.ReturnStatement) and isinstance(child.expression, T.Literal) and child.expression.value == "null" for _, child in method):
                    self.nullReturns.add(method.name)
        arities = {}
        for name, arity in self.signature:
            arities.setdefault(name, set()).add(arity)
        self.rustNames = {}
        for (name, arity) in self.signature:
            base = VIRTUAL.get(name, camel(name))
            self.rustNames[(name, arity)] = base if len(arities[name]) == 1 or arity == min(arities[name]) else f"{base}{arity}"

    def resolve(self, klassName, key):
        for owner in self.chain(klassName):
            if owner in self.impls.get(key, []):
                return owner
        return None

    def virtual(self, key):
        return len(self.impls.get(key, [])) > 1 or key[0] in VIRTUAL


class Scope:
    def __init__(self, parent=None):
        self.vars = {}
        self.parent = parent

    def lookup(self, name):
        if name in self.vars:
            return self.vars[name]
        return self.parent.lookup(name) if self.parent else None

    def declare(self, name, jtype, kind="local"):
        self.vars[name] = (jtype, kind)


class Emitter:
    def __init__(self, program, klass, struct="Piece"):
        self.program, self.klass, self.struct = program, klass, struct
        self.lines = []
        self.todo = set()
        self.renames = {}
        self.mutated = set()
        self.forUpdate = []
        self.breakables = []
        self.nullable = set()
        self.nulled = set()
        self.returns = "void"

    # types

    def componentTypes(self):
        return self.program.componentNames | {"StructureComponent", "StructureTFComponent", "StructureTFStrongholdComponent"}

    def rustType(self, jtype, param=False):
        if jtype in PRIMITIVE:
            return PRIMITIVE[jtype]
        if jtype == "Block":
            return "Blk"
        if jtype == "ChunkCoordinates":
            return "[i32; 3]"
        if jtype == "StructureBoundingBox":
            return "StructureBox"
        if jtype == "Random":
            return "&Random" if param else "Random"
        if jtype == "World":
            return "&GenWorld"
        if jtype == "String":
            return "&'static str"
        if jtype == "TFTreasure":
            return "Table"
        if jtype in STONES:
            return "Stones"
        if jtype.startswith("StructureTFDecorator") or jtype.startswith("StructureDecorator"):
            return "Deco"
        if jtype == "TFMaze":
            return "Rc<Maze>"
        if jtype.endswith("[]"):
            inner = jtype[:-2]
            if inner in self.componentTypes():
                return "&mut Pieces" if param else "Pieces"
            inner = self.rustType(inner)
            return f"&[{inner}]" if param else f"Vec<{inner}>"
        if jtype in self.componentTypes():
            return "&Piece" if param else "Rc<Piece>"
        self.todo.add(jtype)
        return jtype

    def default(self, jtype):
        if jtype in ("float", "double"):
            return "0.0"
        if jtype in PRIMITIVE:
            return "false" if jtype == "boolean" else "0"
        if jtype in STONES:
            return "Stones::stronghold()"
        return {"Block": "AIR", "ChunkCoordinates": "[0, 0, 0]", "StructureBoundingBox": "StructureBox::empty()",
                "TFTreasure": "Table::Basement"}.get(jtype, "Default::default()")

    def fieldKind(self, jtype):
        if jtype == "Random":
            return "plain"
        if jtype.endswith("[]"):
            return "vec"
        if jtype == "TFMaze":
            return "rc"
        if jtype in self.componentTypes():
            return "piece"
        return "cell"

    # expressions return (rust, java type)

    def expr(self, node, scope, want=None):
        rust, jtype = self.exprInner(node, scope)
        return self.coerce(rust, jtype, want, node), jtype

    def coerce(self, rust, jtype, want, node=None):
        if want is None or jtype is None or want == jtype:
            return rust
        if want in WIDTH and jtype in WIDTH and NUMERIC_RUST[want] != NUMERIC_RUST[jtype]:
            if isinstance(node, T.Literal) and re.fullmatch(r"-?\d+", rust) and want in ("float", "double"):
                return rust + ".0"
            if isinstance(node, T.Literal) and re.fullmatch(r"-?\d+", rust) and want == "long":
                return rust
            return f"({rust}) as {NUMERIC_RUST[want]}"
        return rust

    def literal(self, value):
        if value in ("true", "false"):
            return value, "boolean"
        if value == "null":
            return "None", "null"
        if value.startswith('"'):
            return value, "String"
        if value.startswith("'"):
            return f"({value} as i32)", "char"
        text = value.replace("_", "")
        lower = text.lower()
        if lower.startswith("0x"):
            return "0x" + text[2:].rstrip("lL").upper(), "long" if lower.endswith("l") else "int"
        if lower.endswith("l"):
            return text[:-1], "long"
        if lower.endswith("f"):
            number = text[:-1]
            return (number if "." in number else number + ".0").replace(".e", ".0e"), "float"
        if lower.endswith("d"):
            number = text[:-1]
            return (number if "." in number else number + ".0"), "double"
        if "." in text or "e" in lower:
            return (text + "0" if text.endswith(".") else text), "double"
        return text, "int"

    def unary(self, rust, prefix, postfix):
        for op in reversed(prefix or []):
            if op in ("!", "~"):
                rust = f"!{rust}"
            elif op == "-":
                rust = f"-{rust}"
            elif op in ("++", "--"):
                self.todo.add("prefix increment")
                rust = f"{{ {rust} {op[0]}= 1; {rust} }}"
        for op in postfix or []:
            self.todo.add("postfix increment in expression")
            rust = f"{{ let old = {rust}; {rust} {op[0]}= 1; old }}"
        return rust

    def exprInner(self, node, scope):
        if isinstance(node, T.SuperConstructorInvocation):
            return self.superConstructor(node, scope), None
        if isinstance(node, T.ExplicitConstructorInvocation):
            args = node.arguments or []
            items = self.args(args, scope)
            ctor = self.pickConstructor(self.klass, args, scope)
            items = self.args(args, scope, [typeName(p.type) for p in ctor.parameters] if ctor else None)
            return f"init{self.constructorSuffix(self.klass, ctor)}({', '.join(['this'] + items)})", None
        if isinstance(node, T.Literal):
            rust, jtype = self.literal(node.value)
            if getattr(node, "prefix_operators", None):
                rust = self.unary(rust, getattr(node, "prefix_operators", None), None)
            return rust, jtype
        if isinstance(node, T.BinaryOperation):
            rust, jtype = self.binary(node, scope)
            ops = getattr(node, "prefix_operators", None)
            return self.unary(f"({rust})" if ops else rust, ops, None), jtype
        if isinstance(node, T.TernaryExpression):
            cond, _ = self.expr(node.condition, scope, "boolean")
            yesType = self.typeOf(node.if_true, scope)
            noType = self.typeOf(node.if_false, scope)
            jtype = yesType
            if yesType in WIDTH and noType in WIDTH and WIDTH[noType] > WIDTH[yesType]:
                jtype = noType
            yes, _ = self.expr(node.if_true, scope, jtype if jtype in WIDTH else None)
            no, _ = self.expr(node.if_false, scope, jtype if jtype in WIDTH else None)
            return f"if {cond} {{ {yes} }} else {{ {no} }}", jtype
        if isinstance(node, T.Cast):
            target = typeName(node.type)
            inner, jtype = self.expr(node.expression, scope)
            rust = inner
            if target in NUMERIC_RUST and (jtype is None or NUMERIC_RUST.get(jtype) != NUMERIC_RUST[target]):
                rust = f"({inner}) as {NUMERIC_RUST[target]}"
            jtype = target if target not in ("StructureComponent",) else jtype or target
            if getattr(node, "selectors", None):
                rust, jtype = self.applySelectors(f"({rust})", jtype, node.selectors, scope)
            return self.unary(rust, getattr(node, "prefix_operators", None), None), jtype
        if isinstance(node, T.Assignment):
            return self.assignment(node, scope), None
        if isinstance(node, T.ClassCreator):
            return self.creator(node, scope)
        if isinstance(node, T.ArrayCreator):
            inner = typeName(node.type)
            if node.initializer is not None:
                items = [self.expr(i, scope, inner if inner in WIDTH else None)[0] for i in node.initializer.initializers]
                return f"vec![{', '.join(items)}]", inner + "[]"
            size, _ = self.expr(node.dimensions[0], scope, "int")
            if len(node.dimensions) > 1 and node.dimensions[1] is not None:
                size2, _ = self.expr(node.dimensions[1], scope, "int")
                return f"vec![vec![{self.default(inner)}; ({size2}) as usize]; ({size}) as usize]", inner + "[][]"
            return f"vec![{self.default(inner)}; ({size}) as usize]", inner + "[]"
        if isinstance(node, T.ArrayInitializer):
            items = [self.expr(i, scope)[0] for i in node.initializers]
            return f"vec![{', '.join(items)}]", None
        if isinstance(node, T.LambdaExpression):
            self.todo.add("lambda")
            return "todo_lambda()", None
        if isinstance(node, T.This):
            rust, jtype = self.applySelectors("this", self.klass.name, node.selectors or [], scope)
            return self.unary(rust, getattr(node, "prefix_operators", None), getattr(node, "postfix_operators", None)), jtype
        if isinstance(node, T.MemberReference):
            return self.member(node, scope)
        if isinstance(node, T.MethodInvocation):
            return self.invocation(node, scope)
        if isinstance(node, T.SuperMethodInvocation):
            return self.superCall(node, scope)
        if isinstance(node, T.ClassReference):
            return typeName(node.type), None
        self.todo.add(type(node).__name__)
        return f"todo_{type(node).__name__}()", None

    def typeOf(self, node, scope):
        saved = set(self.todo)
        _, jtype = self.exprInner(node, scope)
        self.todo = saved
        return jtype

    def binary(self, node, scope):
        op = node.operator
        if op in ("&&", "||"):
            left, _ = self.expr(node.operandl, scope, "boolean")
            right, _ = self.expr(node.operandr, scope, "boolean")
            return f"{self.paren(left, node.operandl)} {op} {self.paren(right, node.operandr)}", "boolean"
        if op == "instanceof":
            left, _ = self.expr(node.operandl, scope)
            test = self.instanceOf("piece", node.operandr)
            if self.isNullable(node.operandl, scope) or self.mayBeNull(node.operandl):
                return f"{self.paren(left, node.operandl)}.as_ref().is_some_and(|piece| {test})", "boolean"
            return self.instanceOf(left, node.operandr), "boolean"
        leftType = self.typeOf(node.operandl, scope)
        rightType = self.typeOf(node.operandr, scope)
        if op in ("==", "!=") and "null" in (leftType, rightType):
            other = node.operandl if rightType == "null" else node.operandr
            if self.isNullable(other, scope):
                rust = self.localName(other.member)
                return f"{rust}.{'is_none' if op == '==' else 'is_some'}()", "boolean"
            # java nulls only these, the port's values are never missing
            return ("false" if op == "==" else "true"), "boolean"
        if op in ("==", "!=") and (leftType in self.componentTypes() or rightType in self.componentTypes()):
            left, _ = self.expr(node.operandl, scope)
            right, _ = self.expr(node.operandr, scope)
            return f"{'' if op == '==' else '!'}same(&{left}, &{right})", "boolean"
        common = None
        if leftType in WIDTH and rightType in WIDTH and op not in ("<<", ">>", ">>>"):
            common = leftType if WIDTH[leftType] >= WIDTH[rightType] else rightType
        left, _ = self.expr(node.operandl, scope, common)
        right, _ = self.expr(node.operandr, scope, common)
        left, right = self.paren(left, node.operandl), self.paren(right, node.operandr)
        if op == ">>>":
            return f"(({left}) as u32 >> {right}) as i32", leftType
        if op in ("==", "!=", "<", ">", "<=", ">="):
            return f"{left} {op} {right}", "boolean"
        if op in ("&", "|", "^") and leftType == "boolean":
            return f"{left} {op} {right}", "boolean"
        return f"{left} {op} {right}", common or leftType

    def mayBeNull(self, node):
        if isinstance(node, T.Literal) and node.value == "null":
            return True
        if isinstance(node, T.This) and len(node.selectors or []) == 1:
            node = node.selectors[0]
        if isinstance(node, T.Cast):
            node = node.expression
        if isinstance(node, T.MethodInvocation) and not node.selectors:
            return node.member in ("func_74883_a", "findIntersectingExcluding") or node.member in self.program.nullReturns
        if isinstance(node, T.TernaryExpression):
            return self.mayBeNull(node.if_true) or self.mayBeNull(node.if_false)
        return False

    def nulledNames(self, method):
        names = set()
        for _, child in method:
            if isinstance(child, T.Assignment) and isinstance(child.expressionl, T.MemberReference) and not child.expressionl.qualifier and self.mayBeNull(child.value):
                names.add(child.expressionl.member)
        return names

    def isNullable(self, node, scope):
        return isinstance(node, T.MemberReference) and not node.qualifier and not node.selectors and node.member in self.nullable and scope.lookup(node.member) is not None

    def isThis(self, node):
        return isinstance(node, T.This) and not node.selectors

    def instanceOf(self, left, typeNode):
        target = typeNode.name if hasattr(typeNode, "name") else str(typeNode)
        if target in BASES or target not in self.program.classes:
            return "true" if target in BASES else f"todo_instanceof_{target}()"
        kinds = self.program.descendants(target) or [target]
        return f"matches!({left}.m_kind, {' | '.join('Kind::' + kindName(k) for k in kinds)})"

    def paren(self, rust, node):
        if isinstance(node, (T.BinaryOperation, T.TernaryExpression, T.Cast, T.Assignment)) or rust.startswith("-") or rust.startswith("!"):
            return f"({rust})"
        return rust

    def creator(self, node, scope):
        name = node.type.name
        if node.type.sub_type is not None:
            name = node.type.sub_type.name
        args = node.arguments or []
        if name == "ChunkCoordinates":
            items = [self.expr(a, scope, "int")[0] for a in args]
            if len(items) == 1:
                return items[0], "ChunkCoordinates"
            return f"[{', '.join(items)}]", "ChunkCoordinates"
        if name == "StructureBoundingBox":
            items = [self.expr(a, scope, "int")[0] for a in args]
            if len(items) == 6:
                return f"StructureBox::new([{items[0]}, {items[1]}, {items[2]}], [{items[3]}, {items[4]}, {items[5]}])", name
            if len(items) == 1:
                return items[0], name
            return f"StructureBox::flat({', '.join(items)})", name
        if name == "Random":
            seed = self.expr(args[0], scope, "long")[0] if args else "0"
            return f"Random::withSeed({seed})", "Random"
        if name in ("ArrayList", "LinkedList"):
            return "Vec::new()", None
        if name == "TFMaze":
            items = [self.expr(a, scope, "int")[0] for a in args]
            return f"Rc::new(Maze::new({', '.join(items)}))", "TFMaze"
        if self.rustType(name) == "Deco":
            return f"Deco::{snake(name)}()", name
        if name in STONES:
            return f"Stones::{snake(name)}()", name
        if name in self.program.componentNames:
            klass = self.program.classes[name]
            ctor = self.pickConstructor(klass, args, scope)
            params = [typeName(p.type) for p in ctor.parameters] if ctor else [None] * len(args)
            items = [self.argument(a, scope, p) for a, p in zip(args, params)]
            suffix = self.constructorSuffix(klass, ctor)
            return f"{klass.module}::new{suffix}({', '.join(items)})", name
        self.todo.add(f"new {name}")
        items = [self.expr(a, scope)[0] for a in args]
        return f"{name}::new({', '.join(items)})", name

    def pickConstructor(self, klass, args, scope=None):
        if isinstance(args, int):
            return next((c for c in klass.constructors if len(c.parameters) == args), None)
        candidates = [c for c in klass.constructors if len(c.parameters) == len(args)]
        if len(candidates) <= 1:
            return candidates[0] if candidates else None
        types = [self.typeOf(a, scope) for a in args]

        def score(ctor):
            total = 0
            for param, argType in zip(ctor.parameters, types):
                paramType = typeName(param.type)
                if paramType == argType or (paramType in WIDTH and argType in WIDTH) or argType in (None, "null"):
                    total += 1
            return total
        return max(candidates, key=score)

    def constructorSuffix(self, klass, ctor):
        if isinstance(ctor, int):
            ctor = self.pickConstructor(klass, ctor)
        if ctor is None:
            return ""
        arity = len(ctor.parameters)
        arities = sorted({len(c.parameters) for c in klass.constructors})
        suffix = "" if len(arities) <= 1 or arity == arities[0] else str(arity)
        twins = [c for c in klass.constructors if len(c.parameters) == arity]
        return suffix + ("b" * twins.index(ctor) if len(twins) > 1 else "")

    def argument(self, node, scope, ptype):
        if isinstance(node, T.Literal) and node.value == "null" and ptype in ("World", "Random"):
            return "world" if ptype == "World" else "random"
        rust, jtype = self.expr(node, scope, ptype if ptype in WIDTH else None)
        if jtype is None:
            return rust
        if self.isNullable(node, scope) and ptype in self.componentTypes():
            return f"{rust}.piece()"
        if jtype.endswith("[]") and jtype[:-2] in self.componentTypes() and isinstance(node, T.MemberReference) and not node.selectors:
            found = scope.lookup(node.member)
            return rust if found is not None and found[1] == "param" else f"&mut {rust}"
        if isinstance(node, T.ClassCreator) and ptype in self.componentTypes() and jtype in self.componentTypes():
            return f"&{rust}"
        refType = ptype is None or ptype.endswith("[]") or ptype in self.componentTypes()
        if refType and (jtype.endswith("[]") or jtype in self.componentTypes()) and not self.isThis(node) and not isinstance(node, T.ClassCreator):
            if rust == "list" or rust.startswith("&"):
                return rust
            if jtype in self.componentTypes() and isinstance(node, T.MethodInvocation):
                return f"&{rust}"
            return f"&{rust}"
        if jtype == "Random" and self.isOwned(node, scope):
            return f"&{rust}"
        return rust

    def isOwned(self, node, scope):
        if isinstance(node, T.MemberReference) and not node.qualifier:
            found = scope.lookup(node.member)
            return found is not None and found[1] in ("local", "loop")
        return False

    # member chains

    def member(self, node, scope):
        qualifier = node.qualifier or ""
        name = node.member
        if qualifier == "Blocks":
            return f'blk("{BLOCKS.get(name.replace("field_", ""), "UNKNOWN_" + name)}")', "Block"
        if qualifier == "TFBlocks":
            return f'blk("tf:{name}")', "Block"
        if qualifier == "TFTreasure":
            return f"Table::{TREASURE.get(name, name)}", "TFTreasure"
        if qualifier == "TFFeature":
            return f"Feature::{FEATURES.get(name, name)}", "TFFeature"
        if qualifier == "Integer" and name in ("MIN_VALUE", "MAX_VALUE"):
            return f"i32::{name[:3]}", "int"
        if qualifier and qualifier[0].isupper() and "." not in qualifier:
            if qualifier in self.program.classes:
                static = self.program.classes[qualifier].statics.get(name)
                prefix = "" if qualifier == self.klass.name else f"{self.program.classes[qualifier].module}::"
                if static is not None:
                    return f"{prefix}{self.constName(name)}", static[0]
                return f"{prefix}{name}", None
            self.todo.add(f"{qualifier}.{name}")
            return f"{qualifier}::{name}", None
        if qualifier:
            parts = qualifier.split(".")
            base, jtype = self.name(parts[0], scope)
            selectors = [T.MemberReference(member=p) for p in parts[1:]] + [T.MemberReference(member=name)] + list(node.selectors or [])
        else:
            base, jtype = self.name(name, scope)
            selectors = list(node.selectors or [])
        if selectors and name in self.nullable and not qualifier:
            base += ".piece()"
        if qualifier and qualifier.split(".")[0] in self.nullable:
            base += ".piece()"
        rust, jtype = self.applySelectors(base, jtype, selectors, scope)
        return self.unary(rust, getattr(node, "prefix_operators", None), getattr(node, "postfix_operators", None)), jtype

    def constName(self, name):
        return name if name.isupper() else re.sub(r"(?<=[a-z0-9])(?=[A-Z])", "_", name).upper()

    def name(self, name, scope):
        found = scope.lookup(name)
        if found is not None:
            return self.localName(name), found[0]
        if name == "field_74887_e":
            return "this.m_box.get()", "StructureBoundingBox"
        if name == "field_74885_f":
            return "this.m_mode.get()", "int"
        if name == "field_74886_g":
            return "this.m_depth.get()", "int"
        field = self.program.fieldOf(self.klass.name, name)
        if field is not None:
            return self.fieldRead("this", field[0], field[1]), field[1]
        if name == "deco" and self.struct == "Piece":
            return "this.m_deco.get()", "StructureTFDecorator"
        if name == "spawnListIndex" and self.struct == "Piece":
            return "this.m_spawnListIndex.get()", "int"
        for owner in [self.klass.name] + self.program.chain(self.klass.name):
            static = self.program.classes[owner].statics.get(name) if owner in self.program.classes else None
            if static is not None:
                prefix = "" if owner == self.klass.name else f"{self.program.classes[owner].module}::"
                return prefix + self.constName(name), static[0]
        return self.localName(name), None

    def localName(self, name):
        if name in self.renames:
            return self.renames[name]
        if name in KEYWORDS:
            return f"r#{name}"
        return name

    def fieldRead(self, owner, name, jtype):
        kind = self.fieldKind(jtype)
        if kind == "cell":
            return f"{owner}.m_{name}.get()"
        if kind == "rc":
            return f"{owner}.m_{name}.borrow().clone()"
        if kind == "plain":
            return f"{owner}.m_{name}"
        if kind == "piece":
            return f"{owner}.m_{name}.borrow().piece().clone()"
        return f"{owner}.m_{name}.borrow()"

    def applySelectors(self, rust, jtype, selectors, scope):
        for sel in selectors:
            if isinstance(sel, T.MemberReference):
                rust, jtype = self.selectField(rust, jtype, sel.member, scope)
            elif isinstance(sel, T.MethodInvocation):
                rust, jtype = self.callOn(rust, jtype, sel, scope)
            elif isinstance(sel, T.ArraySelector):
                index, _ = self.expr(sel.index, scope, "int")
                index = index if re.fullmatch(r"\d+", index) else f"({index}) as usize"
                rust = f"{rust}[{index}]"
                jtype = jtype[:-2] if jtype and jtype.endswith("[]") else None
            else:
                self.todo.add(type(sel).__name__)
        return rust, jtype

    def selectField(self, rust, jtype, name, scope):
        if name == "field_73012_v":
            return f"{rust}.rand()", "Random"
        if name == "field_74887_e":
            return f"{rust}.m_box.get()", "StructureBoundingBox"
        if name == "field_74885_f":
            return f"{rust}.m_mode.get()", "int"
        if name == "field_74886_g":
            return f"{rust}.m_depth.get()", "int"
        if name in BOX_FIELDS:
            return f"{rust}.{BOX_FIELDS[name]}", "int"
        if name in COORD_FIELDS:
            return f"{rust}{COORD_FIELDS[name]}", "int"
        if name == "length" and (jtype is None or jtype.endswith("[]")):
            return f"({rust}.len() as i32)", "int"
        if jtype and self.rustType(jtype) == "Deco":
            blocks = {"blockID", "accentID", "stairID", "fenceID", "pillarID", "platformID", "floorID", "roofID"}
            return f"{rust}.m_{name}", "Block" if name in blocks else "BlockSelector" if name == "randomBlocks" else "int"
        if name == "deco":
            return f"{rust}.m_deco.get()", "StructureTFDecorator"
        owner = jtype if jtype in self.program.classes else self.klass.name
        field = self.program.fieldOf(owner, name)
        if field is None and name in self.program.fields:
            field = (name, self.program.fields[name][0])
        if field is not None:
            return self.fieldRead(rust, field[0], field[1]), field[1]
        if owner in self.program.classes:
            static = self.program.classes[owner].statics.get(name)
            if static is not None:
                return f"{self.program.classes[owner].module}::{self.constName(name)}", static[0]
        self.todo.add(f"field {name} on {jtype}")
        return f"{rust}.m_{name}", None

    # calls

    def args(self, nodes, scope, params=None):
        params = params or [None] * len(nodes)
        return [self.argument(a, scope, p) for a, p in zip(nodes, params)]

    def invocation(self, node, scope):
        qualifier = node.qualifier or ""
        statics = ("Math", "MathHelper", "StructureComponent", "StructureBoundingBox", "StructureTFComponent", "FMLLog",
                   "TFFeature", "TFCreatures", "MapGenStructureIO", "Integer", "String", "Arrays")
        if not qualifier:
            rust, jtype = self.callOn("this", self.klass.name, node, scope, implicit=True)
        elif qualifier in statics or (qualifier in self.program.classes and scope.lookup(qualifier) is None):
            rust, jtype = self.staticCall(qualifier, node, scope)
        else:
            parts = qualifier.split(".")
            base, baseType = self.name(parts[0], scope)
            if parts[0] in self.nullable and scope.lookup(parts[0]) is not None:
                base += ".piece()"
            base, baseType = self.applySelectors(base, baseType, [T.MemberReference(member=p) for p in parts[1:]], scope)
            rust, jtype = self.callOn(base, baseType, node, scope)
        rust, jtype = self.applySelectors(rust, jtype, node.selectors or [], scope)
        return self.unary(rust, getattr(node, "prefix_operators", None), getattr(node, "postfix_operators", None)), jtype

    def staticCall(self, qualifier, node, scope):
        name = node.member
        args = node.arguments or []
        if qualifier == "Math":
            values = [self.expr(a, scope) for a in args]
            if name == "abs":
                return f"{self.paren(values[0][0], args[0])}.abs()", values[0][1]
            if name in ("max", "min"):
                common = max((v[1] or "int" for v in values), key=lambda t: WIDTH.get(t, 0))
                items = [self.coerce(v[0], v[1], common, a) for v, a in zip(values, args)]
                return f"{self.paren(items[0], args[0])}.{name}({items[1]})", common
            if name == "round":
                return f"{self.paren(values[0][0], args[0])}.round() as i32", "int"
            if name in ("sqrt", "sin", "cos", "atan2", "floor", "ceil", "pow", "toRadians", "toDegrees"):
                items = [self.coerce(v[0], v[1], "double", a) for v, a in zip(values, args)]
                if name == "atan2":
                    return f"{self.paren(items[0], args[0])}.atan2({items[1]})", "double"
                if name == "pow":
                    return f"{self.paren(items[0], args[0])}.powf({items[1]})", "double"
                method = {"toRadians": "to_radians", "toDegrees": "to_degrees"}.get(name, name)
                return f"{self.paren(items[0], args[0])}.{method}()", "double"
        if qualifier == "MathHelper" and name in MATH:
            rustName, jtype = MATH[name]
            values = [self.expr(a, scope, "double" if rustName in ("floor", "ceil") else "float")[0] for a in args]
            return f"{rustName}({', '.join(values)})", jtype
        if qualifier in ("StructureComponent", "StructureTFComponent") and name in STATIC_HELPERS:
            rustName, jtype = STATIC_HELPERS[name]
            items = [self.argument(a, scope, "int" if rustName.startswith("boxFor") else None) for a in args]
            return f"{rustName}({', '.join(items)})", jtype
        if qualifier == "StructureBoundingBox":
            items = [self.expr(a, scope, "int")[0] for a in args]
            if name == "func_78889_a":
                return f"vanillaBox({', '.join(items)})", "StructureBoundingBox"
            if name == "func_78887_a":
                return "StructureBox::empty()", "StructureBoundingBox"
        if qualifier == "TFCreatures" and name == "getSpawnerNameFor":
            return f"mob({self.expr(args[0], scope)[0]})", "String"
        if qualifier == "FMLLog":
            return "()", None
        if qualifier in self.program.classes:
            klass = self.program.classes[qualifier]
            if name in STATIC_HELPERS:
                rustName, jtype = STATIC_HELPERS[name]
                return f"{rustName}({', '.join(self.args(args, scope))})", jtype
            method = next((m for m in klass.methods if m.name == name and len(m.parameters) == len(args)), None)
            params = [typeName(p.type) for p in method.parameters] if method else None
            items = self.args(args, scope, params)
            prefix = "" if klass.name == self.klass.name else f"{klass.module}::"
            return f"{prefix}{camel(name)}({', '.join(items)})", typeName(method.return_type) if method else None
        self.todo.add(f"{qualifier}.{name}")
        return f"{qualifier}::{name}({', '.join(self.args(args, scope))})", None

    def callOn(self, rust, jtype, node, scope, implicit=False):
        name = node.member
        args = node.arguments or []
        if jtype == "World" or rust == "world":
            rustName, ret = WORLD_METHODS.get(name, (name, None))
            if name not in WORLD_METHODS:
                self.todo.add(f"world.{name}")
            return f"world.{rustName}({', '.join(self.args(args, scope))})", ret
        if jtype == "Random":
            rustName, ret = RANDOM_METHODS.get(name, (name, None))
            if name == "nextInt" and not args:
                rustName = "nextInt"
            values = [self.expr(a, scope, "long" if name == "setSeed" else "int")[0] for a in args]
            return f"{rust}.{rustName}({', '.join(values)})", ret
        if jtype == "StructureBoundingBox":
            rustName, ret = BOX_METHODS.get(name, (name, None))
            if name not in BOX_METHODS:
                self.todo.add(f"box.{name}")
            items = [self.argument(a, scope, "int") for a in args]
            if rustName in ("intersects", "expand"):
                items = [f"&{items[0]}"]
            if rustName == "shift":
                items = [f"[{', '.join(items)}]"]
            if rustName in BOX_MUTATING and rust.endswith(".m_box.get()"):
                owner = rust[: -len(".m_box.get()")]
                return f"{owner}.boxed(|bounds| bounds.{rustName}({', '.join(items)}))", ret
            return f"{rust}.{rustName}({', '.join(items)})", ret
        if jtype and jtype.endswith("[]") and name in ("add", "size", "get", "isEmpty", "remove", "contains", "clear", "set", "indexOf", "addAll"):
            return self.listCall(rust, jtype, name, args, scope)
        if jtype in STONES:
            if name == "func_75062_a":
                return f"{rust}.select({', '.join(self.args(args, scope))})", None
            if name == "func_151561_a":
                return f"{rust}.block()", "Block"
            if name == "func_75064_b":
                return f"{rust}.meta()", "int"
        if jtype in self.program.classes and jtype not in self.componentTypes() or (self.struct != "Piece" and rust == "this"):
            klass = self.program.classes.get(jtype) or self.klass
            method = next((m for m in klass.methods if m.name == name and len(m.parameters) == len(args)), None)
            params = [typeName(p.type) for p in method.parameters] if method else None
            items = self.args(args, scope, params)
            ret = typeName(method.return_type) if method else None
            rustName = plainName(klass, name, len(args))
            if method is not None and "static" in method.modifiers:
                prefix = "" if klass is self.klass else f"{klass.module}::"
                return f"{prefix}{rustName}({', '.join(items)})", ret
            return f"{rust}.{rustName}({', '.join(items)})", ret
        if rust == "this" or jtype in self.componentTypes():
            if name in HELPERS:
                rustName, ret = HELPERS[name]
                if name in ("placeTreasureAtCurrentPosition", "placeTreasureRotated") and len(args) == 8:
                    rustName += "Trapped"
                if name == "getLadderMeta" and len(args) == 2:
                    rustName = "ladderMetaTurned"
                items = self.args(args, scope)
                items = ["random" if index == 1 and item == "None" else item for index, item in enumerate(items)]
                if name in ("func_151551_a",):
                    items[3] = self.expr(args[3], scope, "float")[0]
                return f"{rust}.{rustName}({', '.join(items)})", ret
            key = (name, len(args))
            owner = jtype if jtype in self.program.componentNames else self.klass.name
            if key in self.program.signature:
                method = self.program.signature[key]
                params = [typeName(p.type) for p in method.parameters]
                items = self.args(args, scope, params)
                ret = typeName(method.return_type) if method.return_type else None
                rustName = self.program.rustNames[key]
                implementer = self.program.resolve(owner, key) if owner in self.program.componentNames else None
                if self.program.virtual(key) or implementer is None:
                    return f"{rust}.{rustName}({', '.join(items)})", ret
                target = self.program.classes[implementer].module
                receiver = rust if rust == "this" else f"&{rust}"
                return f"{target}::{rustName}({', '.join([receiver] + items)})", ret
            static = self.staticOf(owner, name, len(args))
            if static is not None:
                klass, method = static
                params = [typeName(p.type) for p in method.parameters]
                items = self.args(args, scope, params)
                prefix = "" if klass.name == self.klass.name else f"{klass.module}::"
                return f"{prefix}{camel(name)}({', '.join(items)})", typeName(method.return_type) if method.return_type else None
            if name in STATIC_HELPERS:
                rustName, ret = STATIC_HELPERS[name]
                return f"{rustName}({', '.join(self.args(args, scope))})", ret
        if name == "equals" and args:
            other, _ = self.expr(args[0], scope)
            return f"{rust} == {other}", "boolean"
        self.todo.add(f"call {name} on {jtype}")
        return f"{rust}.{camel(name)}({', '.join(self.args(args, scope))})", None

    def staticOf(self, owner, name, arity):
        for klassName in [owner] + self.program.chain(owner):
            klass = self.program.classes.get(klassName)
            if klass is None:
                continue
            for method in klass.methods:
                if method.name == name and len(method.parameters) == arity and "static" in method.modifiers:
                    return klass, method
        return None

    def listCall(self, rust, jtype, name, args, scope):
        inner = jtype[:-2]
        mutable = rust[: -len(".borrow()")] + ".borrow_mut()" if rust.endswith(".borrow()") else rust
        if name == "add":
            if len(args) == 1:
                value, vtype = self.expr(args[0], scope)
                if self.isNullable(args[0], scope):
                    value = f"Rc::clone({value}.piece())"
                elif vtype in self.componentTypes() and isinstance(args[0], T.MemberReference):
                    value = f"Rc::clone(&{value})"
                elif self.isThis(args[0]):
                    self.todo.add("list.add(this)")
                return f"{mutable}.push({value})", None
            index, _ = self.expr(args[0], scope, "int")
            value, _ = self.expr(args[1], scope)
            return f"{mutable}.insert(({index}) as usize, {value})", None
        if name == "size":
            return f"({rust}.len() as i32)", "int"
        if name == "indexOf":
            value, _ = self.expr(args[0], scope)
            return f"{rust}.iter().position(|item| *item == {value}).map_or(-1, |index| index as i32)", "int"
        if name == "addAll":
            value, _ = self.expr(args[0], scope)
            return f"{mutable}.extend({value}.iter().cloned())", None
        if name == "isEmpty":
            return f"{rust}.is_empty()", "boolean"
        if name == "get":
            index, _ = self.expr(args[0], scope, "int")
            value = f"{rust}[({index}) as usize]"
            return (f"Rc::clone(&{value})" if inner in self.componentTypes() else value), inner
        if name == "clear":
            return f"{mutable}.clear()", None
        if name == "contains":
            value, _ = self.expr(args[0], scope)
            return f"{rust}.contains(&{value})", "boolean"
        if name == "remove":
            index, _ = self.expr(args[0], scope, "int")
            return f"{mutable}.remove(({index}) as usize)", inner
        index, _ = self.expr(args[0], scope, "int")
        value, _ = self.expr(args[1], scope)
        return f"{mutable}[({index}) as usize] = {value}", None

    def superCall(self, node, scope):
        args = node.arguments or []
        key = (node.member, len(args))
        parent = self.klass.parent
        if node.member in HELPERS:
            rustName, ret = HELPERS[node.member]
            return f"this.{rustName}({', '.join(self.args(args, scope))})", ret
        implementer = self.program.resolve(parent, key) if parent in self.program.componentNames else None
        if implementer is None:
            if node.member in VIRTUAL:
                return "()", None
            self.todo.add(f"super.{node.member}")
            return f"todo_super_{node.member}()", None
        method = self.program.signature[key]
        params = [typeName(p.type) for p in method.parameters]
        items = self.args(args, scope, params)
        ret = typeName(method.return_type) if method.return_type else None
        return f"{self.program.classes[implementer].module}::{self.program.rustNames[key]}({', '.join(['this'] + items)})", ret

    def superConstructor(self, node, scope):
        parent = self.klass.parent
        args = node.arguments or []
        if parent in BASES or parent is None or parent not in self.program.classes:
            if args:
                depth, _ = self.expr(args[0], scope, "int")
                return f"this.m_depth.set({depth})"
            return "()"
        klass = self.program.classes[parent]
        ctor = self.pickConstructor(klass, args, scope)
        params = [typeName(p.type) for p in ctor.parameters] if ctor else None
        items = self.args(args, scope, params)
        return f"{klass.module}::init{self.constructorSuffix(klass, ctor)}({', '.join(['this'] + items)})"

    # assignment

    def assignment(self, node, scope):
        op = node.type
        target = node.expressionl
        targetType = self.typeOf(target, scope)
        valueType = self.typeOf(node.value, scope)
        if op != "=" and targetType in WIDTH and valueType in WIDTH and NUMERIC_RUST[valueType] != NUMERIC_RUST[targetType] and op not in ("<<=", ">>=", ">>>="):
            current, _ = self.expr(target, scope)
            value, _ = self.expr(node.value, scope)
            wide = valueType if WIDTH[valueType] > WIDTH[targetType] else targetType
            mixed = f"({self.coerce(current, targetType, wide)} {op[:-1]} {self.paren(self.coerce(value, valueType, wide, node.value), node.value)})"
            return self.store(target, "=", self.coerce(mixed, wide, targetType), scope)
        value, _ = self.expr(node.value, scope, targetType if targetType in WIDTH else None)
        if valueType == "null" and targetType and targetType not in self.componentTypes():
            value = self.default(targetType)
        if targetType in self.componentTypes() and isinstance(node.value, T.MemberReference) and valueType in self.componentTypes():
            value = f"Rc::clone(&{value})"
        return self.store(target, op, value, scope)

    def store(self, target, op, value, scope):
        if isinstance(target, T.This):
            selectors = list(target.selectors or [])
            base = "this"
        elif isinstance(target, T.MemberReference):
            qualifier = target.qualifier or ""
            parts = [p for p in qualifier.split(".") if p] + [target.member]
            selectors = [T.MemberReference(member=p) for p in parts[1:]] + list(target.selectors or [])
            first = parts[0]
            special = ("field_74887_e", "field_74885_f", "field_74886_g", "deco", "spawnListIndex")
            if scope.lookup(first) is None and (self.program.fieldOf(self.klass.name, first) or first in special):
                base = "this"
                selectors = [T.MemberReference(member=first)] + selectors
            else:
                base = self.localName(first)
                if not selectors:
                    if first in self.nullable and value != "None" and not value.startswith("intersecting"):
                        value = f"Some({value})"
                    return f"{base} {op} {value}"
        else:
            self.todo.add("assign target")
            return f"todo_assign({value})"
        found = scope.lookup(base)
        jtype = self.klass.name if base == "this" else (found[0] if found else None)
        last = selectors[-1]
        if len(selectors) >= 2 and isinstance(selectors[-2], T.MemberReference) and selectors[-2].member == "field_74887_e" and isinstance(last, T.MemberReference) and last.member in BOX_FIELDS:
            owner, _ = self.applySelectors(base, jtype, selectors[:-2], scope)
            return f"{owner}.boxed(|bounds| bounds.{BOX_FIELDS[last.member]} {op} {value})"
        rust, jtype = self.applySelectors(base, jtype, selectors[:-1], scope)
        if isinstance(last, T.ArraySelector):
            index, _ = self.expr(last.index, scope, "int")
            index = index if re.fullmatch(r"\d+", index) else f"({index}) as usize"
            if rust.endswith(".borrow()"):
                rust = rust[: -len(".borrow()")] + ".borrow_mut()"
            rust = rust.replace(".borrow()[", ".borrow_mut()[", 1)
            return f"{rust}[{index}] {op} {value}"
        name = last.member
        simple = {"field_74887_e": "m_box", "field_74885_f": "m_mode", "field_74886_g": "m_depth"}
        if name in simple:
            if op == "=":
                return f"{rust}.{simple[name]}.set({value})"
            return f"{rust}.{simple[name]}.set({rust}.{simple[name]}.get() {op[:-1]} {value})"
        if name in BOX_FIELDS:
            return f"{rust}.{BOX_FIELDS[name]} {op} {value}"
        if name in COORD_FIELDS:
            return f"{rust}{COORD_FIELDS[name]} {op} {value}"
        if jtype and self.rustType(jtype) == "Deco":
            return f"{rust}.m_{name} {op} {value}"
        owner = jtype if jtype in self.program.classes else self.klass.name
        field = self.program.fieldOf(owner, name)
        if field is None:
            if name == "deco":
                field = ("deco", "StructureTFDecorator")
            elif name == "spawnListIndex":
                field = ("spawnListIndex", "int")
            elif jtype and self.rustType(jtype) == "Deco":
                return f"{rust}.m_{name} {op} {value}"
            else:
                field = (name, self.program.fields.get(name, ("int",))[0])
        if self.fieldKind(field[1]) == "cell":
            if op == "=":
                return f"{rust}.m_{field[0]}.set({value})"
            return f"{rust}.m_{field[0]}.set({rust}.m_{field[0]}.get() {op[:-1]} {self.paren(value, None)})"
        if self.fieldKind(field[1]) == "plain" and value.startswith("Random::withSeed("):
            return f"{rust}.m_{field[0]}.setSeed({value[len('Random::withSeed('):-1]})"
        if self.fieldKind(field[1]) == "piece":
            return f"*{rust}.m_{field[0]}.borrow_mut() = {'None' if value == 'None' else 'Some(' + value + ')'}"
        if self.fieldKind(field[1]) == "rc":
            return f"*{rust}.m_{field[0]}.borrow_mut() = {value}"
        return f"*{rust}.m_{field[0]}.borrow_mut() {op} {value}"

    # statements

    def emit(self, text, depth):
        self.lines.append("    " * depth + text)

    def block(self, statements, scope, depth):
        inner = Scope(scope)
        for statement in statements or []:
            self.statement(statement, inner, depth)

    def assignedNames(self, node):
        names = set()
        for _, child in node:
            if isinstance(child, T.Assignment) and isinstance(child.expressionl, T.MemberReference) and not child.expressionl.qualifier:
                names.add(child.expressionl.member)
            if isinstance(child, T.Assignment) and isinstance(child.expressionl, T.MemberReference) and child.expressionl.qualifier:
                names.add(child.expressionl.qualifier.split(".")[0])
            if isinstance(child, T.MemberReference) and (set(getattr(child, "prefix_operators", None) or []) | set(getattr(child, "postfix_operators", None) or [])) & {"++", "--"}:
                names.add(child.qualifier.split(".")[0] if child.qualifier else child.member)
            if isinstance(child, T.MethodInvocation) and child.qualifier and child.member in ("add", "remove", "clear", "setSeed", "func_78886_a", "func_78888_b", "set"):
                names.add(child.qualifier.split(".")[0])
        return names

    def statement(self, node, scope, depth):
        if isinstance(node, T.LocalVariableDeclaration):
            jtype = typeName(node.type)
            for declarator in node.declarators:
                rustType = self.rustType(jtype)
                mutable = "mut " if declarator.name in self.mutated else ""
                local = self.localName(declarator.name)
                init = declarator.initializer
                if init is None:
                    scope.declare(declarator.name, jtype)
                    if declarator.name in self.nulled and jtype in self.componentTypes() | {"ChunkCoordinates", "StructureBoundingBox"}:
                        rustType = f"Option<{rustType}>"
                        self.nullable.add(declarator.name)
                    self.emit(f"let {mutable}{local}: {rustType};", depth)
                    continue
                nullable = self.mayBeNull(init) or declarator.name in self.nulled
                value, valueType = self.expr(init, scope, jtype if jtype in WIDTH else None)
                if jtype in self.componentTypes() and isinstance(init, T.MemberReference) and valueType in self.componentTypes():
                    value = f"Rc::clone(&{value})"
                if jtype in self.componentTypes() and self.isThis(init):
                    self.todo.add("local = this")
                if jtype.endswith("[]") and value.endswith(".borrow()"):
                    value += ".clone()"
                if nullable:
                    rustType = f"Option<{rustType}>"
                    self.nullable.add(declarator.name)
                scope.declare(declarator.name, jtype)
                self.emit(f"let {mutable}{local}: {rustType} = {value};", depth)
            return
        if isinstance(node, T.StatementExpression):
            expression = node.expression
            if isinstance(expression, T.MemberReference) and (getattr(expression, "postfix_operators", None) or getattr(expression, "prefix_operators", None)):
                op = (getattr(expression, "postfix_operators", None) or getattr(expression, "prefix_operators", None))[0]
                plain = T.MemberReference(qualifier=expression.qualifier, member=expression.member, selectors=expression.selectors, prefix_operators=[], postfix_operators=[])
                rust = self.store(plain, op[0] + "=", "1", scope)
            elif isinstance(expression, T.This) and getattr(expression, "postfix_operators", None):
                plain = T.This(selectors=expression.selectors, prefix_operators=[], postfix_operators=[])
                rust = self.store(plain, getattr(expression, "postfix_operators", None)[0][0] + "=", "1", scope)
            else:
                rust, _ = self.expr(expression, scope)
            if rust == "()":
                return
            self.emit(rust + ";", depth)
            return
        if isinstance(node, T.BlockStatement):
            self.emit("{", depth)
            self.block(node.statements, scope, depth + 1)
            self.emit("}", depth)
            return
        if isinstance(node, T.IfStatement):
            cond, _ = self.expr(node.condition, scope, "boolean")
            self.emit(f"if {cond}", depth)
            self.body(node.then_statement, scope, depth)
            other = node.else_statement
            while other is not None:
                if isinstance(other, T.IfStatement):
                    cond, _ = self.expr(other.condition, scope, "boolean")
                    self.emit(f"else if {cond}", depth)
                    self.body(other.then_statement, scope, depth)
                    other = other.else_statement
                else:
                    self.emit("else", depth)
                    self.body(other, scope, depth)
                    other = None
            return
        if isinstance(node, T.WhileStatement):
            cond, _ = self.expr(node.condition, scope, "boolean")
            self.emit("loop" if cond == "true" else f"while {cond}", depth)
            self.loopBody(node.body, scope, depth)
            return
        if isinstance(node, T.DoStatement):
            cond, _ = self.expr(node.condition, scope, "boolean")
            self.emit("loop", depth)
            self.emit("{", depth)
            self.breakables.append(None)
            self.inline(node.body, scope, depth + 1)
            self.breakables.pop()
            self.emit(f"if !({cond})", depth + 1)
            self.emit("{", depth + 1)
            self.emit("break;", depth + 2)
            self.emit("}", depth + 1)
            self.emit("}", depth)
            return
        if isinstance(node, T.ForStatement):
            self.forStatement(node, scope, depth)
            return
        if isinstance(node, T.ReturnStatement):
            if node.expression is None:
                self.emit("return;", depth)
                return
            value, valueType = self.expr(node.expression, scope, self.returns if self.returns in WIDTH else None)
            local = self.isNullable(node.expression, scope)
            if getattr(self, "optional", False):
                if valueType != "null" and not local and not self.mayBeNull(node.expression):
                    if self.returns in self.componentTypes() and isinstance(node.expression, T.MemberReference):
                        value = f"Rc::clone(&{value})"
                    value = f"Some({value})"
            elif local:
                value = f"Rc::clone({value}.piece())"
            elif self.returns in self.componentTypes() and isinstance(node.expression, T.MemberReference) and valueType in self.componentTypes():
                value = f"Rc::clone(&{value})"
            if self.returns.endswith("[]") and value.endswith(".borrow()"):
                value += ".clone()"
            self.emit(f"return {value};", depth)
            return
        if isinstance(node, T.BreakStatement):
            if node.goto:
                self.emit(f"break '{node.goto};", depth)
            elif self.breakables and self.breakables[-1]:
                self.emit(f"break '{self.breakables[-1]};", depth)
            else:
                self.emit("break;", depth)
            return
        if isinstance(node, T.ContinueStatement):
            if node.goto:
                self.emit(f"continue '{node.goto};", depth)
            elif self.forUpdate and self.forUpdate[-1]:
                self.emit(f"break '{self.forUpdate[-1]};", depth)
            else:
                self.emit("continue;", depth)
            return
        if isinstance(node, T.SwitchStatement):
            self.switch(node, scope, depth)
            return
        if isinstance(node, T.ThrowStatement):
            self.emit('panic!("structure generation failed");', depth)
            return
        if isinstance(node, T.TryStatement):
            self.block(node.block, scope, depth)
            return
        if type(node) is T.Statement:
            return
        self.todo.add(type(node).__name__)
        self.emit(f"// todo {type(node).__name__}", depth)

    def loopBody(self, node, scope, depth):
        self.breakables.append(None)
        self.body(node, scope, depth)
        self.breakables.pop()

    def body(self, node, scope, depth):
        self.emit("{", depth)
        self.inline(node, scope, depth + 1)
        self.emit("}", depth)

    def inline(self, node, scope, depth):
        if isinstance(node, T.BlockStatement):
            self.block(node.statements, scope, depth)
        elif node is not None:
            self.statement(node, Scope(scope), depth)

    def forStatement(self, node, scope, depth):
        control = node.control
        inner = Scope(scope)
        if isinstance(control, T.EnhancedForControl):
            var = control.var.declarators[0].name
            jtype = typeName(control.var.type)
            iterable, iterType = self.expr(control.iterable, scope)
            inner.declare(var, jtype, "loop")
            self.emit(f"for {self.localName(var)} in {iterable}.clone()", depth)
            self.forUpdate.append(None)
            self.loopBody(node.body, inner, depth)
            self.forUpdate.pop()
            return
        simple = self.simpleRange(control, scope, node.body)
        if simple is not None:
            var, rng = simple
            inner.declare(var, "int", "loop")
            self.emit(f"for {self.localName(var)} in {rng}", depth)
            self.forUpdate.append(None)
            self.loopBody(node.body, inner, depth)
            self.forUpdate.pop()
            return
        self.emit("{", depth)
        inits = control.init if isinstance(control.init, list) else ([control.init] if control.init is not None else [])
        for init in inits:
            if isinstance(init, T.VariableDeclaration):
                declaration = T.LocalVariableDeclaration(type=init.type, declarators=init.declarators, modifiers=set(), annotations=[])
                for declarator in init.declarators:
                    self.mutated.add(declarator.name)
                self.statement(declaration, inner, depth + 1)
            else:
                self.emit(self.expr(init, inner)[0] + ";", depth + 1)
        cond = self.expr(control.condition, inner, "boolean")[0] if control.condition is not None else "true"
        self.emit("loop" if cond == "true" else f"while {cond}", depth + 1)
        self.emit("{", depth + 1)
        hasContinue = any(isinstance(child, T.ContinueStatement) and not child.goto for _, child in node.body)
        self.breakables.append(None)
        if hasContinue and control.update:
            label = f"step{depth}"
            self.forUpdate.append(label)
            self.emit(f"'{label}: {{", depth + 2)
            self.inline(node.body, inner, depth + 3)
            self.emit("}", depth + 2)
            self.forUpdate.pop()
        else:
            self.forUpdate.append(None)
            self.inline(node.body, inner, depth + 2)
            self.forUpdate.pop()
        self.breakables.pop()
        for update in control.update or []:
            self.statement(T.StatementExpression(expression=update), inner, depth + 2)
        self.emit("}", depth + 1)
        self.emit("}", depth)

    def simpleRange(self, control, scope, body):
        if not isinstance(control.init, T.VariableDeclaration) or len(control.init.declarators) != 1:
            return None
        if typeName(control.init.type) not in ("int", "byte"):
            return None
        var = control.init.declarators[0].name
        if body is not None and var in self.assignedNames(body):
            return None
        cond = control.condition
        if not isinstance(cond, T.BinaryOperation) or not isinstance(cond.operandl, T.MemberReference) or cond.operandl.member != var or cond.operandl.qualifier or cond.operandl.selectors:
            return None
        if not control.update or len(control.update) != 1:
            return None
        update = control.update[0]
        step = None
        if isinstance(update, T.MemberReference) and update.member == var:
            ops = (getattr(update, "postfix_operators", None) or []) + (getattr(update, "prefix_operators", None) or [])
            step = 1 if "++" in ops else -1 if "--" in ops else None
        elif isinstance(update, T.Assignment) and isinstance(update.expressionl, T.MemberReference) and update.expressionl.member == var and update.type in ("+=", "-=") and isinstance(update.value, T.Literal) and re.fullmatch(r"\d+", update.value.value):
            amount = int(update.value.value)
            step = amount if update.type == "+=" else -amount
        if step is None:
            return None
        start = self.expr(control.init.declarators[0].initializer, scope, "int")[0]
        end = self.expr(cond.operandr, scope, "int")[0]
        op = cond.operator
        if step > 0 and op in ("<", "<="):
            rng = f"{self.paren(start, None) if ' ' in start else start}..{'=' if op == '<=' else ''}{self.paren(end, cond.operandr)}"
            return var, (f"({rng}).step_by({step})" if step != 1 else rng)
        if step < 0 and op in (">", ">="):
            low = end if op == ">=" else f"{self.paren(end, cond.operandr)} + 1"
            rng = f"({low}..={start}).rev()"
            if step != -1:
                self.todo.add("reverse stepped loop")
                return var, f"{rng}.step_by({-step})"
            return var, rng
        return None

    def switch(self, node, scope, depth):
        value, jtype = self.expr(node.expression, scope)
        early = False
        for case in node.cases:
            body = case.statements[:-1] if case.statements and isinstance(case.statements[-1], T.BreakStatement) else case.statements
            for statement in body:
                for _, child in statement:
                    if isinstance(child, T.BreakStatement) and not child.goto:
                        early = True
        caseLabel = f"case{depth}" if early else None
        if caseLabel:
            self.emit(f"'{caseLabel}: {{", depth)
            depth += 1
        self.breakables.append(caseLabel)
        self.emit(f"match {value}", depth)
        self.emit("{", depth)
        pending = []
        hasDefault = False
        for index, case in enumerate(node.cases):
            labels = []
            for label in case.case:
                if isinstance(label, T.MemberReference) and not label.qualifier and jtype and jtype not in WIDTH:
                    labels.append(f"{jtype}::{label.member}")
                else:
                    labels.append(self.expr(label, scope)[0])
            if not case.case:
                labels = ["_"]
                hasDefault = True
            pending += labels
            statements = list(case.statements)
            if not statements:
                continue
            ends = isinstance(statements[-1], (T.BreakStatement, T.ReturnStatement, T.ContinueStatement, T.ThrowStatement))
            if isinstance(statements[-1], T.BreakStatement) and not statements[-1].goto:
                statements = statements[:-1]
            if not ends and index != len(node.cases) - 1:
                self.todo.add("switch fallthrough")
                self.emit("// todo falls through to the next case", depth + 1)
            self.emit(f"{' | '.join(pending)} =>", depth + 1)
            self.emit("{", depth + 1)
            self.block(statements, scope, depth + 2)
            self.emit("}", depth + 1)
            pending = []
        if not hasDefault:
            self.emit("_ => {}", depth + 1)
        self.emit("}", depth)
        self.breakables.pop()
        if caseLabel:
            self.emit("}", depth - 1)

    # functions

    def params(self, method, scope, mutated):
        out = []
        for param in method.parameters:
            jtype = typeName(param.type)
            name = self.localName(param.name)
            if jtype == "World":
                name = "world"
            elif jtype == "Random" and param.name in ("rand", "par2Random", "random", "par1Random"):
                name = "random"
            elif re.fullmatch(r"par\d+StructureBoundingBox|structureboundingbox|par\d+StructureBoundingBox\d*", param.name):
                name = "sbb"
            scope.declare(param.name, jtype, "param")
            if name != param.name:
                self.renames[param.name] = name
            mut = "mut " if param.name in mutated and (jtype in PRIMITIVE or jtype in ("ChunkCoordinates", "StructureBoundingBox")) else ""
            out.append(f"{mut}{name}: {self.rustType(jtype, param=True)}")
        return out

    def function(self, name, method, isStatic=False):
        scope = Scope()
        self.renames = {}
        self.mutated = self.assignedNames(method)
        self.forUpdate = []
        self.breakables = []
        self.nullable = set()
        self.nulled = self.nulledNames(method)
        params = self.params(method, scope, self.mutated)
        self.returns = typeName(method.return_type) if getattr(method, "return_type", None) is not None else "void"
        self.optional = method.name in self.program.nullReturns and self.returns != "void"
        ret = "" if self.returns == "void" else f" -> {self.rustType(self.returns)}"
        if self.optional:
            ret = f" -> Option<{self.rustType(self.returns)}>"
        head = [] if isStatic else [f"this: &{self.struct}"]
        self.emit(f"// {self.klass.name}.{method.name}", 0)
        self.emit(f"pub fn {name}({', '.join(head + params)}){ret}", 0)
        self.emit("{", 0)
        self.block(method.body or [], scope, 1)
        self.emit("}", 0)
        self.emit("", 0)

    def constructor(self, ctor):
        name = f"init{self.constructorSuffix(self.klass, ctor)}"
        scope = Scope()
        self.renames = {}
        self.mutated = self.assignedNames(ctor)
        self.forUpdate = []
        self.breakables = []
        self.nullable = set()
        self.nulled = self.nulledNames(ctor)
        self.returns = "void"
        params = self.params(ctor, scope, self.mutated)
        self.emit(f"// {self.klass.name} constructor", 0)
        self.emit(f"pub fn {name}({', '.join([f'this: &{self.struct}'] + params)})", 0)
        self.emit("{", 0)
        statements = list(ctor.body or [])
        if statements and isinstance(statements[0], T.StatementExpression) and isinstance(statements[0].expression, (T.SuperConstructorInvocation, T.ExplicitConstructorInvocation)):
            self.statement(statements.pop(0), scope, 1)
        for field, init in self.klass.initializers:
            jtype = self.klass.fields[field]
            value, _ = self.expr(init, scope, jtype if jtype in WIDTH else None)
            real = self.klass.renamed.get(field, field)
            kind = self.fieldKind(jtype)
            self.emit(f"this.m_{real}.set({value});" if kind == "cell" else f"*this.m_{real}.borrow_mut() = {value};", 1)
        self.block(statements, scope, 1)
        self.emit("}", 0)
        self.emit("", 0)


def componentModule(program, klass):
    emitter = Emitter(program, klass)
    for name, (jtype, init) in klass.statics.items():
        if init is None or not (jtype in PRIMITIVE or jtype.endswith("[]")):
            continue
        value, _ = emitter.expr(init, Scope(), jtype if jtype in WIDTH else None)
        rustType = emitter.rustType(jtype)
        if jtype.endswith("[]"):
            rustType = f"&[{emitter.rustType(jtype[:-2])}]"
            value = value.replace("vec![", "&[", 1)
        emitter.emit(f"pub const {emitter.constName(name)}: {rustType} = {value};", 0)
    if klass.statics:
        emitter.emit("", 0)
    if not klass.abstract:
        for ctor in klass.constructors:
            suffix = emitter.constructorSuffix(klass, ctor)
            emitter.renames = {}
            params = emitter.params(ctor, Scope(), set())
            names = [p.split(":")[0].replace("mut ", "") for p in params]
            emitter.emit(f"pub fn new{suffix}({', '.join(params)}) -> Rc<Piece>", 0)
            emitter.emit("{", 0)
            emitter.emit(f"let this: Piece = Piece::blank(Kind::{kindName(klass.name)});", 1)
            emitter.emit(f"init{suffix}({', '.join(['&this'] + names)});", 1)
            emitter.emit("Rc::new(this)", 1)
            emitter.emit("}", 0)
            emitter.emit("", 0)
    for ctor in klass.constructors:
        emitter.constructor(ctor)
    for method in klass.methods:
        if method.body is None:
            continue
        if "static" in method.modifiers:
            emitter.function(camel(method.name), method, isStatic=True)
            continue
        emitter.function(program.rustNames[(method.name, len(method.parameters))], method)
    return emitter


def mazeModule(program, klass):
    emitter = Emitter(program, klass, struct="Maze")
    for name, (jtype, init) in klass.statics.items():
        if init is None:
            continue
        value, _ = emitter.expr(init, Scope(), jtype if jtype in WIDTH else None)
        emitter.emit(f"pub const {emitter.constName(name)}: {emitter.rustType(jtype)} = {value};", 0)
    emitter.emit("", 0)
    emitter.emit("pub struct Maze", 0)
    emitter.emit("{", 0)
    for name, jtype in klass.fields.items():
        kind = emitter.fieldKind(jtype)
        rustType = emitter.rustType(jtype)
        emitter.emit(f"pub m_{name}: {rustType}," if kind == "plain" else f"pub m_{name}: {'Cell' if kind == 'cell' else 'RefCell'}<{rustType}>,", 1)
    emitter.emit("}", 0)
    emitter.emit("", 0)
    emitter.emit("impl Maze", 0)
    emitter.emit("{", 0)
    start = len(emitter.lines)
    emitter.emit("pub fn empty() -> Maze", 0)
    emitter.emit("{", 0)
    def initial(jtype):
        kind = emitter.fieldKind(jtype)
        return {"cell": f"Cell::new({emitter.default(jtype)})", "plain": "Random::withSeed(0)"}.get(kind, "RefCell::new(Vec::new())")
    inits = ", ".join(f"m_{name}: {initial(jtype)}" for name, jtype in klass.fields.items())
    emitter.emit(f"Maze {{ {inits} }}", 1)
    emitter.emit("}", 0)
    emitter.emit("", 0)
    for ctor in klass.constructors:
        suffix = emitter.constructorSuffix(klass, ctor)
        emitter.renames = {}
        params = emitter.params(ctor, Scope(), set())
        names = [p.split(":")[0].replace("mut ", "") for p in params]
        emitter.emit(f"pub fn new{suffix}({', '.join(params)}) -> Maze", 0)
        emitter.emit("{", 0)
        emitter.emit("let maze: Maze = Maze::empty();", 1)
        emitter.emit(f"maze.init{suffix}({', '.join(names)});", 1)
        emitter.emit("maze", 1)
        emitter.emit("}", 0)
        emitter.emit("", 0)
        emitter.constructor(ctor)
    for method in klass.methods:
        if method.body is not None:
            emitter.function(plainName(klass, method.name, len(method.parameters)), method, isStatic="static" in method.modifiers)
    body = emitter.lines[start:]
    body = [re.sub(r"^pub fn (\w+)\(this: &Maze(, )?", r"pub fn \1(&self\2", line) for line in body]
    body = [re.sub(r"\bthis\b", "self", line) for line in body]
    emitter.lines[start:] = ["    " + line if line else line for line in body]
    emitter.emit("}", 0)
    return emitter


def pieceModule(program):
    emitter = Emitter(program, program.components[0])
    out = ["// generated by tools/structgen.py, every StructureComponent class as one kind of piece",
           "use crate::world::structures::prelude::*;", "", "#[derive(Clone, Copy, PartialEq, Eq, Debug)]", "pub enum Kind", "{"]
    out += [f"    {kindName(k.name)}," for k in program.components]
    out += ["}", "", "pub struct Piece", "{", "    pub m_kind: Kind,", "    pub m_box: Cell<StructureBox>,",
            "    pub m_mode: Cell<i32>,", "    pub m_depth: Cell<i32>,", "    pub m_deco: Cell<Deco>,",
            "    pub m_spawnListIndex: Cell<i32>,"]
    for name, (jtype, _) in sorted(program.fields.items()):
        kind = emitter.fieldKind(jtype)
        rustType = "Option<Rc<Piece>>" if kind == "piece" else emitter.rustType(jtype)
        out.append(f"    pub m_{name}: {'Cell' if kind == 'cell' else 'RefCell'}<{rustType}>,")
    out += ["}", "", "impl Piece", "{", "    pub fn blank(kind: Kind) -> Piece", "    {", "        Piece {",
            "            m_kind: kind,", "            m_box: Cell::new(StructureBox::empty()),",
            "            m_mode: Cell::new(0),", "            m_depth: Cell::new(0),",
            "            m_deco: Cell::new(Deco::default()),", "            m_spawnListIndex: Cell::new(0),"]
    for name, (jtype, _) in sorted(program.fields.items()):
        kind = emitter.fieldKind(jtype)
        if kind == "cell":
            out.append(f"            m_{name}: Cell::new({emitter.default(jtype)}),")
        elif kind == "rc":
            out.append(f"            m_{name}: RefCell::new(Rc::new(Maze::empty())),")
        elif kind == "piece":
            out.append(f"            m_{name}: RefCell::new(None),")
        else:
            out.append(f"            m_{name}: RefCell::new(Vec::new()),")
    out += ["        }", "    }"]
    for key in sorted(program.signature, key=lambda k: program.rustNames[k]):
        if not program.virtual(key):
            continue
        method = program.signature[key]
        rustName = program.rustNames[key]
        emitter.renames = {}
        params = emitter.params(method, Scope(), set())
        names = [p.split(":")[0].replace("mut ", "") for p in params]
        returns = typeName(method.return_type) if method.return_type else "void"
        ret = "" if returns == "void" else f" -> {emitter.rustType(returns)}"
        if key[0] in program.nullReturns and returns != "void":
            ret = f" -> Option<{emitter.rustType(returns)}>"
        out += ["", f"    pub fn {rustName}({', '.join(['&self'] + params)}){ret}", "    {", "        match self.m_kind", "        {"]
        groups = {}
        missing = False
        for klass in program.components:
            implementer = program.resolve(klass.name, key)
            if implementer is None:
                missing = True
            else:
                groups.setdefault(implementer, []).append(klass.name)
        for implementer, kinds in groups.items():
            pattern = " | ".join("Kind::" + kindName(k) for k in kinds)
            out.append(f"            {pattern} => {program.classes[implementer].module}::{rustName}({', '.join(['self'] + names)}),")
        if missing:
            if key[0] in VIRTUAL:
                out.append(f"            _ => {'{}' if ret == '' else 'true'},")
            else:
                out.append(f'            _ => unreachable!("{key[0]} on {{:?}}", self.m_kind),')
        out += ["        }", "    }"]
    out.append("}")
    return out


def main():
    source, target = Path(sys.argv[1]), Path(sys.argv[2])
    only = set(sys.argv[sys.argv.index("--only") + 1].split(",")) if "--only" in sys.argv else None
    program = Program(source)
    todo = {}
    packages = {}
    for klass in program.components:
        packages.setdefault(klass.package or "base", []).append(klass.module)
        if only and klass.name not in only:
            continue
        emitter = componentModule(program, klass)
        folder = target / (klass.package or "base")
        folder.mkdir(parents=True, exist_ok=True)
        text = f"// draft from {klass.name}.java by tools/structgen.py\nuse crate::world::structures::prelude::*;\n\n" + "\n".join(emitter.lines).rstrip() + "\n"
        (folder / f"{klass.module}.rs").write_text(text, encoding="utf-8")
        todo[klass.name] = emitter.todo
    packages.setdefault("base", []).append("maze")
    if not only or "TFMaze" in only:
        emitter = mazeModule(program, program.classes["TFMaze"])
        (target / "base").mkdir(parents=True, exist_ok=True)
        text = "// draft from TFMaze.java by tools/structgen.py\nuse crate::world::structures::prelude::*;\n\n" + "\n".join(emitter.lines).rstrip() + "\n"
        (target / "base" / "maze.rs").write_text(text, encoding="utf-8")
        todo["TFMaze"] = emitter.todo
    if not only:
        (target / "piece.rs").write_text("\n".join(pieceModule(program)) + "\n", encoding="utf-8")
        for package, mods in packages.items():
            (target / package / "mod.rs").write_text("".join(f"pub mod {m};\n" for m in sorted(mods)), encoding="utf-8")
        lines = [f"pub use super::{package}::{{{', '.join(sorted(mods))}}};" for package, mods in sorted(packages.items())]
        print("\n".join(lines))
    for name, items in sorted(todo.items()):
        if items:
            print(f"{name}: {', '.join(sorted(items))}")


if __name__ == "__main__":
    main()
