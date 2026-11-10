use crate::world::features::Feature;

// TFFeature.createHintBook, the explorer's notes a hint kobold carries, one per feature that guards its pieces
pub struct Hint
{
    pub m_feature: Option<Feature>,
    pub m_item: &'static str,
    pub m_title: &'static str,
    pub m_pages: &'static [&'static str],
}

pub const AUTHOR: &str = "A Forgotten Explorer";

pub const HINTS: [Hint; 9] = [
    Hint
    {
        m_feature: Some(Feature::LichTower),
        m_item: "hint_book_lich_tower",
        m_title: r#"Notes on a Pointy Tower"#,
        m_pages: &[
            r#"§8[[An explorer's notebook, gnawed on by monsters]]§0

I have begun examining the strange aura surrounding this tower. The bricks of the tower are protected by a curse, stronger than any I've seen before. The magic from the curse"#,
            r#"is boiling off into the surrounding area.

In my homeland I would have many options for dealing with this magic, but here my supplies are limited. I shall have to research..."#,
            r#"§8[[Many entries later]]§0

A breakthrough!  In my journeys I sighted a huge snake-like monster in a decorated courtyard. Nearby, I picked up a worn down, discarded green scale.

The magic in the scale seems to have the"#,
            r#"curse-breaking properties I need, but the magic is too dim. I may need to acquire a fresher specimen, directly from the creature."#,
            r#"A Forgotten Explorer"#,
            r#"Notes on a Pointy Tower"#,
        ],
    },
    Hint
    {
        m_feature: Some(Feature::Labyrinth),
        m_item: "hint_book_labyrinth",
        m_title: r#"Notes on a Swampy Labyrinth"#,
        m_pages: &[
            r#"§8[[An explorer's notebook, written on waterproof paper]]§0

The mosquitoes in this swamp are vexing, but strange. The vast majority of them seem to have no natural source, nor do they seem to have a role in the local ecology. I have begun to"#,
            r#"suspect that they are some kind of magical curse.

§8[[Next entry]]§0

Now that I have encountered a protection spell on the ruined labyrinth here, I consider my suspicions confirmed. Both the protection"#,
            r#"spell and the mosquitoes are a curse. This curse seems to have a different source from the others I have encountered. I will have to research further...

§8[[Next entry]]§0

The curse seems to"#,
            r#"be of a type too powerful for one being alone to produce. Several wizards working in combination would be necessary.

If one of the wizards stopped contributing, the whole of the curse over the entire swamp would fall. Strangely, "#,
            r#"my divinations do not show signs of any nearby living wizards. I did see something interesting in one of the nearby pointy-roofed towers though..."#,
            r#"A Forgotten Explorer"#,
            r#"Notes on a Swampy Labyrinth"#,
        ],
    },
    Hint
    {
        m_feature: Some(Feature::HydraLair),
        m_item: "hint_book_hydra_lair",
        m_title: r#"Notes on the Fire Swamp"#,
        m_pages: &[
            r#"§8[[An explorer's notebook, written on fireproof paper]]§0

Fire is a trivial obstacle for a master explorer such as myself. I have traversed seas of fire, and swam through oceans of lava. The burning air here is an interesting variation,"#,
            r#"but ultimately no hinderance.

What does stop me though is that I have encountered another protection spell, this time surrounding a mighty creature that must be king of this fire swamp. This is not the first protection spell I have"#,
            r#"encountered, and I am beginning to unravel the mysteries of how they work.

If this spell is like the others, it will be sustained by a powerful creature nearby. Surrounding the fire swamp are several wet swamps, and under those"#,
            r#"swamps are labyrinths full of minotaurs. The logical choice to bind such a spell to would be some sort of powerful minotaur, different in some way from the others that surround it..."#,
            r#"A Forgotten Explorer"#,
            r#"Notes on the Fire Swamp"#,
        ],
    },
    Hint
    {
        m_feature: Some(Feature::Stronghold),
        m_item: "hint_book_stronghold",
        m_title: r#"Notes on a Stronghold"#,
        m_pages: &[
            r#"§8[[An explorer's notebook, written on faintly glowing paper]]§0

The tendrils of darkness surrounding this area are just a manifestation of a protective spell over the entire dark forest. The spell causes blindness, which is quite vexing. I have"#,
            r#"seen several interesting things in the area and would like to keep exploring.

§8[[Next entry]]§0

I have found ruins in the dark forest.  They belong to a stronghold, of a type usually inhabited by knights. Rather than"#,
            r#"knights though, this stronghold is full of goblins. They wear knightly armor, but their behavior is most un-knightly.

§8[[Next entry]]§0

Deep in the ruins, I have found a pedestal. The pedestal seems to be of a type that"#,
            r#"knights would place trophies on to prove their strength.

Killing a powerful creature would seem to weaken the curse on the dark forest, and placing a trophy associated with the creature on the pedestal would likely grant access into the"#,
            r#"main part of the stronghold.

The only creature I have seen so far seen so far of sufficient power is the many-headed beast in the fire swamp. How vexing..."#,
            r#"A Forgotten Explorer"#,
            r#"Notes on a Stronghold"#,
        ],
    },
    Hint
    {
        m_feature: Some(Feature::DarkTower),
        m_item: "hint_book_dark_tower",
        m_title: r#"Notes on a Wooden Tower"#,
        m_pages: &[
            r#"§8[[An explorer's notebook that seems to have survived an explosion]]§0

This tower clearly has mechanisms that are not responding to me. Their magic almost yearns to acknowledge my touch, but it cannot. It is if the devices of the tower are being"#,
            r#"suppressed by a powerful group of beings nearby.

§8[[Next entry]]§0

The magic seems to emanate from deep within the strongholds nearby. It can't come from the goblins, as their magic is charming, but unfocused. There"#,
            r#"must still be some force still active in the strongholds.

§8[[Next entry]]§0

My analysis indicates that it comes from several sources, operating as a group. I will head back to the stronghold after I resupply..."#,
            r#"A Forgotten Explorer"#,
            r#"Notes on a Wooden Tower"#,
        ],
    },
    Hint
    {
        m_feature: Some(Feature::YetiCave),
        m_item: "hint_book_yeti_cave",
        m_title: r#"Notes on an Icy Cave"#,
        m_pages: &[
            r#"§8[[An explorer's notebook, covered in frost]]§0

The blizzard surrounding these snowy lands is unceasing. This is no ordinary snowfall--this is a magical phenomenon. I will have to conduct experiments to find"#,
            r#"what is capable of causing such an effect.

§8[[Next entry]]§0

At the center of the dark forest, where the leaves turn red and the grass dies, there is a wooden tower. The tops of the tower are affixed with"#,
            r#"structures acting as antennae. The antennae are not the source of the snowfall, but serve merely to boost the power of the curse causing it.

A blizzard this intense must be caused by a powerful creature, most likely found"#,
            r#"near the top of the dark forest tower. Stop the creature, and the blizzard will fade."#,
            r#"A Forgotten Explorer"#,
            r#"Notes on an Icy Cave"#,
        ],
    },
    Hint
    {
        m_feature: Some(Feature::IceTower),
        m_item: "hint_book_ice_tower",
        m_title: r#"Notes on an Auroral Fortification"#,
        m_pages: &[
            r#"§8[[An explorer's notebook, caked in ice]]§0

I overcame one blizzard, only to run into this terrible ice storm atop the glacier. My explorations have shown me the splendor of an ice palace, shining with the colors of the polar aurora. It"#,
            r#"all seems protected by some sort of curse.

§8[[Next entry]]§0

I am no novice.  This curse is fed by the power of a creature nearby.  The cause of the curse surrounding the fire swamp was built off the power of the leader of the "#,
            r#"minotaurs nearby.

Surrounding this glacier, there are masses of yetis.  Perhaps the yetis have some sort of leader..."#,
            r#"A Forgotten Explorer"#,
            r#"Notes on an Auroral Fortification"#,
        ],
    },
    Hint
    {
        m_feature: Some(Feature::TrollCave),
        m_item: "hint_book_troll_cave",
        m_title: r#"Notes on an the Highlands"#,
        m_pages: &[
            r#"§8[[An explorer's notebook, damaged by acid]]§0

There seems to be no way to protect myself from the toxic rainstorm surrounding this area. In my brief excursions, I have also encountered another protection spell, similar to the"#,
            r#"others I have witnessed. The spell must be connected to the toxic storm in some way. Further research to follow...

§8[[Next entry]]§0

Such supreme weather magic must be the result of an unequaled weather"#,
            r#"magician. Such a person would likely hide themselves in an extreme environment, far away.

Based on my logic, I would expect to find such a person somewhere on the glacier, perhaps in some sort of fortress there..."#,
            r#"A Forgotten Explorer"#,
            r#"Notes on an the Highlands"#,
        ],
    },
    Hint
    {
        m_feature: None,
        m_item: "hint_book_unexplained",
        m_title: r#"Notes on the Unexplained"#,
        m_pages: &[
            r#"§8[[This book shows signs of having been copied many times]]§0

I cannot explain the field surrounding this structure, but the magic is powerful.  If this curse is like the others, than the answer to unlocking it lies elsewhere.  Perhaps there is "#,
            r#"something I have left undone, or some monster I have yet to defeat. I will have to turn back. I will return to this place later, to see if anything has changed."#,
            r#"A Forgotten Explorer"#,
            r#"Notes on the Unexplained"#,
        ],
    },
];
