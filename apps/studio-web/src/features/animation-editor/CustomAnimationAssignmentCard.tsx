import type { DirectCreatureBaseSlotV1 } from "../animation-mapping/types";
import type { ProjectedCustomAnimationLibraryItemV1 } from "./editing";

export function CustomAnimationAssignmentCard({
  slot,
  item,
  onAssign,
  onClear,
}: {
  slot: DirectCreatureBaseSlotV1;
  item: ProjectedCustomAnimationLibraryItemV1 | null;
  onAssign: () => void;
  onClear: () => void;
}) {
  return (
    <section className="custom-animation-assignment" aria-labelledby="custom-assignment-title">
      <h3 id="custom-assignment-title">Assign Custom to {slot}</h3>
      <p>
        {item
          ? `${item.name} is ${item.status.toLocaleLowerCase("en-US")}.`
          : "Choose a saved Custom animation."}
      </p>
      <button type="button" disabled={!item?.assignable} onClick={onAssign}>
        Assign custom animation
      </button>
      <button type="button" onClick={onClear}>Clear assignment</button>
      <p>Assigning does not remove the clip from the Custom library.</p>
    </section>
  );
}
