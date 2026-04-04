import {
  Empty,
  EmptyHeader,
  EmptyTitle,
  EmptyDescription,
} from "@deck-ui/core";

export function SkillsTab() {
  return (
    <div className="flex-1 flex flex-col">
      <Empty className="border-0">
        <EmptyHeader>
          <EmptyTitle>No skills yet</EmptyTitle>
          <EmptyDescription>
            Skills let your agent learn and remember how to do specific tasks.
          </EmptyDescription>
        </EmptyHeader>
      </Empty>
    </div>
  );
}
