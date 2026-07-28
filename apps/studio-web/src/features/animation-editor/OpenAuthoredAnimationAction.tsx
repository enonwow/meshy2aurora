export function OpenAuthoredAnimationAction({
  clipId,
  onOpen,
}: {
  clipId: string;
  onOpen: (clipId: string) => void;
}) {
  return (
    <button type="button" onClick={() => onOpen(clipId)}>
      Open selected in editor
    </button>
  );
}
