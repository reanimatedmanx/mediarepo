import {Component, Inject} from '@angular/core';
import {MAT_DIALOG_DATA, MatDialogRef} from "@angular/material/dialog";
import {SortKey} from "../../../models/SortKey";
import {CdkDragDrop, moveItemInArray} from "@angular/cdk/drag-drop";

@Component({
  selector: 'app-filter-dialog',
  templateUrl: './filter-dialog.component.html',
  styleUrls: ['./filter-dialog.component.scss']
})
export class FilterDialogComponent {

  public sortEntries: SortKey[] = []

  constructor(public dialogRef: MatDialogRef<FilterDialogComponent>, @Inject(MAT_DIALOG_DATA) data: any) {
    this.sortEntries = data.sortEntries;
  }

  addNewSortKey() {
    const sortKey = new SortKey("FileName", "Ascending", undefined);
    this.sortEntries.push(sortKey)
  }

  public removeSortKey(sortKey: SortKey): void {
    const index = this.sortEntries.indexOf(sortKey);
    this.sortEntries.splice(index, 1);
  }

  public confirmSort(): void {
    this.dialogRef.close(this.sortEntries);
  }

  public cancelSort(): void {
    this.dialogRef.close()
  }

  public onSortEntryDrop(event: CdkDragDrop<SortKey[]>): void {
    moveItemInArray(this.sortEntries, event.previousIndex, event.currentIndex);
  }
}
