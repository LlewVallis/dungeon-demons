const context = new AudioContext();

const output = context.createGain();
output.gain.value = 0.5;
output.connect(context.destination);

class Sound {
  private readonly buffer: Promise<AudioBuffer>;

  constructor(source: string) {
    this.buffer = fetch(source)
      .then((resp) => resp.arrayBuffer())
      .then((buffer) => context.decodeAudioData(buffer));
  }

  async play(): Promise<void> {
    const buffer = await this.buffer;

    const source = context.createBufferSource();
    source.buffer = buffer;
    source.connect(output);
    source.start();
  }
}

const SOUNDS: Record<string, Sound> = {
  shoot_slow: new Sound(require("/assets/shoot-slow.mp3")),
  shoot_fast: new Sound(require("/assets/shoot-fast.mp3")),
  shoot_sniper: new Sound(require("/assets/shoot-sniper.mp3")),
  shoot_shotgun: new Sound(require("/assets/shoot-shotgun.mp3")),
  hit: new Sound(require("/assets/monster-hit.mp3")),
  kill: new Sound(require("/assets/monster-death.mp3")),
  player_hit: new Sound(require("/assets/monster-attack-2.mp3")),
  death: new Sound(require("/assets/down.mp3")),
  round_start: new Sound(require("/assets/round-start.mp3")),
  round_end: new Sound(require("/assets/round-end.mp3")),
  purchase: new Sound(require("/assets/purchase.mp3")),
  pickup: new Sound(require("/assets/pickup.mp3")),
};

async function playSound(name: string) {
  if (!(name in SOUNDS)) {
    console.warn(`Unknown sound: ${name}`);
    return;
  }

  await SOUNDS[name].play();
}

(window as any).dungeonDemonsPlaySound = playSound;
