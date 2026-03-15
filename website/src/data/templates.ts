export interface Template {
  id: string;
  title: string;
  slug: string;
  category: 'fantasy' | 'sci-fi' | 'horror' | 'urban';
  description: string;
  longDescription: string;
  features: string[];
  customization: string[];
  faq: { question: string; answer: string }[];
  related: string[];
}

export const templates: Template[] = [
  {
    id: 'medieval-village',
    title: 'Medieval Fantasy Village',
    slug: 'fantasy/medieval-village',
    category: 'fantasy',
    description: 'Create stunning medieval fantasy worlds with our Medieval Fantasy Village template. AI-powered 3D world creation. Start building for free.',
    longDescription: `
      Step into a living, breathing medieval world. This template provides a complete foundation for RPGs, fantasy simulations, and historical recreations. 
      
      Generate sprawling villages with cobblestone streets, timber-framed houses, and bustling market squares. The template includes a pre-configured terrain engine that sculpts rolling hills and river valleys around your settlement. 
      
      NPCs are pre-programmed with basic daily routines—blacksmiths hammer at their anvils, merchants hawk their wares, and guards patrol the walls. The lighting system features a dynamic day-night cycle, casting long shadows at sunset and illuminating windows with warm candlelight as night falls.
    `,
    features: [
      'Procedural Terrain Generation',
      'NPC Behaviors & Pathfinding',
      'Dynamic Day-Night Lighting',
      'Fire & Smoke Particle Effects',
      'Modular Building System'
    ],
    customization: [
      'Open the Gen console by pressing `~` (tilde key).',
      'Type `add a blacksmith shop near the market` to spawn a new building with functional anvils.',
      'Type `set time to sunset` to instantly preview the dynamic lighting and shadow effects.',
      'Type `populate with 20 villagers` to generate unique NPCs with random schedules.'
    ],
    faq: [
      {
        question: 'Can I export this medieval village to Unity or Unreal?',
        answer: 'Yes. LocalGPT Gen exports to standard .glb and .gltf formats, which are fully compatible with Unity, Unreal Engine 5, Godot, and Blender.'
      },
      {
        question: 'Is this template suitable for mobile games?',
        answer: 'Absolutely. The assets are optimized for performance with LOD (Level of Detail) support, making them suitable for mobile and VR experiences.'
      },
      {
        question: 'Can I customize the architectural style?',
        answer: 'Yes. You can use natural language prompts to shift the style from "High Fantasy" to "Dark Ages" or "Gothic" instantly.'
      }
    ],
    related: ['enchanted-forest', 'japanese-temple', 'winter-wonderland']
  },
  {
    id: 'cyberpunk-city',
    title: 'Cyberpunk Neon City',
    slug: 'urban/cyberpunk-city',
    category: 'urban',
    description: 'Build immersive cyberpunk cities with neon lights, rain-slicked streets, and volumetric fog. The ultimate sci-fi urban template.',
    longDescription: `
      Design the future with the Cyberpunk Neon City template. Perfect for dystopias, high-tech thrillers, and sci-fi shooters.
      
      This template showcases advanced PBR materials with wet-street effects, reflecting the glow of procedural neon signage. Volumetric fog hangs between towering skyscrapers, creating that signature "high tech, low life" atmosphere. 
      
      Includes a dynamic weather system with rain particles that react to light sources. Holographic billboards and flying vehicle traffic lanes bring the verticality of the city to life.
    `,
    features: [
      'Neon Reflections & Global Illumination',
      'Volumetric Fog & Rain Particles',
      'Wet-Street PBR Materials',
      'Holographic Signage Systems',
      'Vertical City Layouts'
    ],
    customization: [
        'Open the Gen console by pressing `~`.',
        'Type `change neon signs to "Neo-Tokyo"` to instantly rebrand the city signage.',
        'Type `increase rain intensity to 80%` to see the wet street shaders react in real-time.',
        'Type `add a flying car traffic lane` to populate the sky with vehicles.'
    ],
    faq: [
      {
        question: 'Does this support real-time ray tracing?',
        answer: 'The template uses high-performance baked lighting and screen-space reflections to approximate ray tracing effects while maintaining high frame rates.'
      },
      {
        question: 'Can I add my own 3D models?',
        answer: 'Yes, you can import any .glb/.gltf models into the scene and the AI will integrate them into the lighting and physics systems.'
      }
    ],
    related: ['modern-city', 'space-station', 'alien-world']
  },
  {
    id: 'haunted-house',
    title: 'Horror Haunted House',
    slug: 'horror/haunted-house',
    category: 'horror',
    description: 'Design terrifying horror games with the Haunted House template. Features flickering lights, fog, and audio triggers.',
    longDescription: `
      Craft your nightmare with the Horror Haunted House template. Ideal for survival horror games, escape rooms, and psychological thrillers.
      
      This template masters the art of tension. It features a flickering light system linked to random event triggers, creaking floorboard audio zones, and atmospheric fog that obscures vision. 
      
      The architecture includes procedural corridors that can shift layout, and "scare zones" where you can script jump scares or environmental storytelling events.
    `,
    features: [
      'Dynamic Flickering Lighting',
      'Audio Trigger Zones',
      'Atmospheric Fog & Dust Particles',
      'Tension-Building Soundscapes',
      'Interactive Doors & hiding spots'
    ],
    customization: [
        'Open the Gen console by pressing `~`.',
        'Type `add a ghost patrol route in the hallway` to generate a basic enemy path.',
        'Type `make the lights flicker when player is near` to add proximity triggers.',
        'Type `change atmosphere to dense fog` to reduce visibility range.'
    ],
    faq: [
      {
        question: 'Is this template multiplayer ready?',
        answer: 'Yes, the state synchronization is built-in, making it perfect for co-op horror games similar to Phasmophobia or Lethal Company.'
      },
      {
        question: 'How do I trigger jump scares?',
        answer: 'The template includes an "Event Trigger" system where you can link player proximity to sound effects, animation, or lighting changes.'
      }
    ],
    related: ['backrooms', 'enchanted-forest', 'medieval-village']
  },
  {
    id: 'enchanted-forest',
    title: 'Enchanted Forest',
    slug: 'fantasy/enchanted-forest',
    category: 'fantasy',
    description: 'Generate magical nature biomes with the Enchanted Forest template. Procedural trees, fireflies, and ambient birdsong.',
    longDescription: `
      Escape to a world of magic with the Enchanted Forest template. Perfect for fantasy adventures, relaxing exploration games, and virtual nature walks.
      
      The terrain engine generates rolling hills, babbling brooks, and ancient trees with procedural variation. At night, the forest comes alive with bioluminescent plants and floating firefly particles. 
      
      Includes a rich ambient audio system with birdsong, wind rustling through leaves, and distant animal calls that react to the time of day.
    `,
    features: [
      'Procedural Vegetation & Trees',
      'Bioluminescent Flora & Fireflies',
      'Ambient Audio Ecosystem',
      'Water Shader with Reflections',
      'Day-Night Cycle'
    ],
    customization: [
        'Open the Gen console by pressing `~`.',
        'Type `change trees to ancient oaks` to swap the vegetation species.',
        'Type `add a stone circle clearing` to create a point of interest.',
        'Type `set time to midnight` to see the bioluminescent plants glow.'
    ],
    faq: [
      {
        question: 'Can I change the season?',
        answer: 'Yes, you can toggle between Spring, Summer, Autumn, and Winter modes to instantly change the foliage colors and weather effects.'
      }
    ],
    related: ['medieval-village', 'winter-wonderland', 'alien-world']
  },
  {
    id: 'space-station',
    title: 'Sci-Fi Space Station',
    slug: 'sci-fi/space-station',
    category: 'sci-fi',
    description: 'Build modular space stations with zero-gravity physics. The Space Station template features airlocks, steam vents, and holograms.',
    longDescription: `
      Launch into orbit with the Sci-Fi Space Station template. The standard for space sims, FPS maps, and extraction shooters.
      
      Features a fully modular corridor and room system that snaps together logicially. The physics engine supports zero-gravity zones where objects float freely. 
      
      Visuals include emergency lighting states (red alert), steam vent particle effects, and functional holographic displays for ship controls.
    `,
    features: [
      'Zero-Gravity Physics Simulation',
      'Modular Corridor System',
      'Emergency Lighting States',
      'Steam & Gas Particle Effects',
      'Holographic UI Displays'
    ],
    customization: [
        'Open the Gen console by pressing `~`.',
        'Type `expand the corridor section by 50 meters` to generate more layout.',
        'Type `trigger red alert mode` to switch all lighting to emergency state.',
        'Type `add a zero-g cargo bay` to create a physics playground.'
    ],
    faq: [
      {
        question: 'Can I simulate hull breaches?',
        answer: 'Yes, the physics system supports decompression events where objects are sucked towards the breach point.'
      }
    ],
    related: ['alien-world', 'cyberpunk-city', 'underwater-world']
  },
  {
    id: 'modern-city',
    title: 'Modern City',
    slug: 'urban/modern-city',
    category: 'urban',
    description: 'Create realistic modern cities with drivable vehicles and pedestrians. The Modern City template is perfect for open-world games.',
    longDescription: `
      Build the metropolis of your dreams. The Modern City template offers a realistic urban sandbox for roleplay servers, racing games, and architectural visualization.
      
      Includes a traffic system with drivable vehicle physics and pedestrian AI that obeys traffic lights. Buildings are generated with enterable interiors or optimized facades for background skylines. 
      
      The road network is procedural, allowing you to sketch layouts that instantly populate with sidewalks, streetlights, and road markings.
    `,
    features: [
      'Vehicle Physics & Traffic System',
      'Pedestrian AI & Crowds',
      'Procedural Road Networks',
      'Enterable Building Interiors',
      'Realistic Glass & Concrete Materials'
    ],
    customization: [
        'Open the Gen console by pressing `~`.',
        'Type `add a central park with a fountain` to break up the concrete.',
        'Type `increase traffic density to high` to fill the streets.',
        'Type `change architecture style to Brutalist` to swap the building assets.'
    ],
    faq: [
      {
        question: 'How large can the city be?',
        answer: 'LocalGPT Gen uses a chunk-based loading system, allowing for virtually infinite city sizes limited only by storage.'
      }
    ],
    related: ['cyberpunk-city', 'cozy-farm', 'medieval-village']
  },
  {
    id: 'underwater-world',
    title: 'Underwater Ocean World',
    slug: 'sci-fi/underwater-world',
    category: 'sci-fi',
    description: 'Explore the depths with the Underwater World template. Features bubble particles, caustics, and swimming physics.',
    longDescription: `
      Dive into the unknown. The Underwater Ocean World template creates immersive sub-aquatic environments for survival games and exploration.
      
      The lighting engine renders realistic water caustics on the ocean floor and volumetric "god rays" piercing from the surface. 
      
      Physics are tuned for buoyancy and drag, giving movement a weighty, fluid feel. Includes schools of procedural fish and swaying kelp forests.
    `,
    features: [
      'Water Caustics & God Rays',
      'Buoyancy & Fluid Physics',
      'Bubble & Silt Particles',
      'Procedural Coral & Kelp',
      'Muffled Underwater Audio'
    ],
    customization: [
        'Open the Gen console by pressing `~`.',
        'Type `add a shipwreck on the cliff edge` to create a mystery.',
        'Type `change water color to tropical blue` for a lighter feel.',
        'Type `add a school of sharks` to introduce danger.'
    ],
    faq: [
      {
        question: 'Does it support swimming mechanics?',
        answer: 'Yes, the player controller automatically switches to swimming mode when submerged, with 6-DOF movement.'
      }
    ],
    related: ['alien-world', 'space-station', 'enchanted-forest']
  },
  {
    id: 'japanese-temple',
    title: 'Japanese Temple & Gardens',
    slug: 'fantasy/japanese-temple',
    category: 'fantasy',
    description: 'Design serene Japanese gardens and temples. Features cherry blossoms, koi ponds, and paper lantern lighting.',
    longDescription: `
      Find your zen. The Japanese Temple & Gardens template captures the beauty of traditional architecture and nature. 
      
      Perfect for social hangout spaces, fighting game stages, and historical adventures. Features include falling cherry blossom petal particles, reflective koi ponds with ripple physics, and soft, warm lighting from paper lanterns. 
      
      The architectural assets respect traditional joinery and roofing styles.
    `,
    features: [
      'Cherry Blossom Particle Systems',
      'Water Ripples & Koi AI',
      'Paper Lantern Lighting',
      'Traditional Architecture Assets',
      'Zen Garden Raking Patterns'
    ],
    customization: [
        'Open the Gen console by pressing `~`.',
        'Type `add a zen rock garden` to generate raked sand patterns.',
        'Type `change season to autumn` to turn the maples red.',
        'Type `add a tea house overlooking the pond`.'
    ],
    faq: [
      {
        question: 'Are the assets historically accurate?',
        answer: 'They are modeled after Heian and Edo period architecture, providing a strong authentic base that can be stylized further.'
      }
    ],
    related: ['medieval-village', 'enchanted-forest', 'cozy-farm']
  },
  {
    id: 'cozy-farm',
    title: 'Cozy Farm Village',
    slug: 'fantasy/cozy-farm',
    category: 'fantasy',
    description: 'Build relaxing farm simulations with the Cozy Farm template. Features crops, weather, and friendly villagers.',
    longDescription: `
      Plant the seeds of a new life. The Cozy Farm Village template is designed for the booming "cozy game" genre.
      
      It includes a farming system with crop growth stages, a weather system that waters plants, and warm, inviting interior lighting. 
      
      Villager NPCs have "friendly" behaviors—waving, gardening, and sitting on benches. The aesthetic leans towards soft, pastel textures and rounded geometry.
    `,
    features: [
      'Crop Growth & Farming Systems',
      'Dynamic Weather & Watering',
      'Warm Interior Lighting',
      'Friendly NPC Interactions',
      'Soft/Pastel Art Style'
    ],
    customization: [
        'Open the Gen console by pressing `~`.',
        'Type `plant a field of corn` to generate crops.',
        'Type `add a barn for livestock` to expand the farm.',
        'Type `change weather to light rain` to water the crops.'
    ],
    faq: [
      {
        question: 'Can I customize the crops?',
        answer: 'Yes, you can define new crop types with unique growth times and models using simple text descriptions.'
      }
    ],
    related: ['medieval-village', 'enchanted-forest', 'japanese-temple']
  },
  {
    id: 'backrooms',
    title: 'Liminal Spaces / Backrooms',
    slug: 'horror/backrooms',
    category: 'horror',
    description: 'Generate infinite liminal spaces. The Backrooms template features procedural rooms, fluorescent hum, and eerie silence.',
    longDescription: `
      Noclip out of reality. The Liminal Spaces / Backrooms template is optimized for generating the unsettled, empty environments that define the genre.
      
      Features an infinite procedural generation algorithm that stitches together "familiar but wrong" rooms. The lighting uses the signature yellow-tinted fluorescent buzz. 
      
      Audio is minimal but oppressive—the hum of lights, distant ventilation, and the sound of your own footsteps.
    `,
    features: [
      'Infinite Procedural Room Generation',
      'Fluorescent Hum Audio',
      'Uncanny/Liminal Lighting',
      'VHS Camera Post-Processing',
      'Non-Euclidean Geometry Support'
    ],
    customization: [
        'Open the Gen console by pressing `~`.',
        'Type `expand the maze complexity` to make navigation harder.',
        'Type `change wallpaper to damp yellow` for that classic look.',
        'Type `add a poolrooms zone` to transition to the aquatic level.'
    ],
    faq: [
      {
        question: 'How do I make it loop?',
        answer: 'The generation engine handles seamless chunk loading to create an effectively infinite space without loading screens.'
      }
    ],
    related: ['haunted-house', 'space-station', 'modern-city']
  },
  {
    id: 'winter-wonderland',
    title: 'Winter Wonderland',
    slug: 'fantasy/winter-wonderland',
    category: 'fantasy',
    description: 'Create snowy landscapes with the Winter Wonderland template. Features snow trails, ice materials, and cozy cabins.',
    longDescription: `
      The chill is in the air. The Winter Wonderland template brings the magic of snow and ice to your projects.
      
      Features a deformable snow system that leaves footprints and trails. Ice materials use subsurface scattering for realistic frozen lakes. 
      
      Atmospheric fog and falling snow particles create depth, while warm point lights in cabins provide a cozy contrast ("hygge").
    `,
    features: [
      'Deformable Snow Trails',
      'Subsurface Ice Materials',
      'Falling Snow Particles',
      'Atmospheric Cold Fog',
      'Cozy Warm/Cold Contrast Lighting'
    ],
    customization: [
        'Open the Gen console by pressing `~`.',
        'Type `start a blizzard` to increase wind and snow density.',
        'Type `add a frozen lake` for skating mechanics.',
        'Type `add a cozy log cabin with smoke`.'
    ],
    faq: [
      {
        question: 'Does the snow accumulate?',
        answer: 'You can enable dynamic accumulation where snow builds up on surfaces over time based on the weather intensity.'
      }
    ],
    related: ['enchanted-forest', 'medieval-village', 'cozy-farm']
  },
  {
    id: 'alien-world',
    title: 'Alien Bioluminescent World',
    slug: 'sci-fi/alien-world',
    category: 'sci-fi',
    description: 'Design exotic alien planets. The Alien World template features strange flora, emissive lighting, and bold colors.',
    longDescription: `
      Explore the galaxy. The Alien Bioluminescent World template is for creating environments that defy Earthly logic.
      
      The terrain generator produces floating islands, twisted rock formations, and crystal caves. Flora is emissive, pulsing with light in the dark. 
      
      The color palette system allows for bold, non-standard skies (purple, green) and multiple suns.
    `,
    features: [
      'Exotic Terrain & Floating Islands',
      'Emissive/Pulsing Flora',
      'Custom Sky & Multi-Sun System',
      'Bold Color Palette Control',
      'Strange Gravity Zones'
    ],
    customization: [
        'Open the Gen console by pressing `~`.',
        'Type `change sky color to deep purple` for an alien atmosphere.',
        'Type `add floating crystal islands` to use verticality.',
        'Type `make all plants glow blue` to change the biome palette.'
    ],
    faq: [
      {
        question: 'Can I create low-gravity worlds?',
        answer: 'Yes, gravity is a global parameter you can adjust, or you can create specific zones with different gravity vectors.'
      }
    ],
    related: ['space-station', 'underwater-world', 'enchanted-forest']
  }
];
