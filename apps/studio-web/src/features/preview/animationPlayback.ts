import * as THREE from "three";

export interface AnimationClipInventoryItem {
  index: number;
  name: string;
  durationSeconds: number;
}

export interface AnimationPlaybackSnapshot {
  selectedClipIndex: number | null;
  playing: boolean;
  loop: boolean;
  timeSeconds: number;
  durationSeconds: number;
}

export function inventoryAnimationClips(clips: readonly THREE.AnimationClip[]): AnimationClipInventoryItem[] {
  return clips.map((clip, index) => ({
    index,
    name: clip.name.trim() || `Unnamed clip ${index + 1}`,
    durationSeconds: Math.max(clip.duration, 0),
  }));
}

export class AnimationPlaybackRuntime {
  private readonly mixer: THREE.AnimationMixer;
  private selectedAction?: THREE.AnimationAction;
  private selectedClipIndex: number | null = null;
  private playing = false;
  private loop = true;

  constructor(
    private readonly root: THREE.Object3D,
    private readonly clips: readonly THREE.AnimationClip[],
  ) {
    this.mixer = new THREE.AnimationMixer(root);
  }

  selectClip(index: number) {
    const clip = this.clips[index];
    if (!clip) throw new RangeError(`Animation clip index ${index} is unavailable`);

    this.mixer.stopAllAction();
    this.selectedClipIndex = index;
    this.selectedAction = this.mixer.clipAction(clip);
    this.configureLoop();
    this.selectedAction.reset().play();
    this.selectedAction.paused = !this.playing;
    return this.snapshot();
  }

  setPlaying(playing: boolean) {
    this.playing = Boolean(this.selectedAction) && playing;
    if (this.selectedAction) {
      if (this.playing && this.selectedAction.time >= this.selectedAction.getClip().duration) {
        this.selectedAction.reset();
      }
      this.selectedAction.paused = !this.playing;
      if (this.playing) this.selectedAction.play();
    }
    return this.snapshot();
  }

  setLoop(loop: boolean) {
    this.loop = loop;
    this.configureLoop();
    return this.snapshot();
  }

  seek(timeSeconds: number) {
    if (this.selectedAction) {
      const duration = Math.max(this.selectedAction.getClip().duration, 0);
      this.selectedAction.time = Math.min(Math.max(timeSeconds, 0), duration);
      this.mixer.update(0);
    }
    return this.snapshot();
  }

  update(deltaSeconds: number) {
    if (this.playing && this.selectedAction) {
      this.mixer.update(Math.max(deltaSeconds, 0));
      if (!this.loop && this.selectedAction.time >= this.selectedAction.getClip().duration) {
        this.playing = false;
        this.selectedAction.paused = true;
      }
    }
    return this.snapshot();
  }

  snapshot(): AnimationPlaybackSnapshot {
    return {
      selectedClipIndex: this.selectedClipIndex,
      playing: this.playing,
      loop: this.loop,
      timeSeconds: this.selectedAction?.time ?? 0,
      durationSeconds: this.selectedAction?.getClip().duration ?? 0,
    };
  }

  dispose() {
    this.mixer.stopAllAction();
    this.mixer.uncacheRoot(this.root);
    this.selectedAction = undefined;
    this.selectedClipIndex = null;
    this.playing = false;
  }

  private configureLoop() {
    if (!this.selectedAction) return;
    this.selectedAction.setLoop(this.loop ? THREE.LoopRepeat : THREE.LoopOnce, this.loop ? Infinity : 1);
    this.selectedAction.clampWhenFinished = !this.loop;
  }
}
