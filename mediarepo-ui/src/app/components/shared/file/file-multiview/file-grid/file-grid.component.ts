import {
    AfterViewInit,
    Component,
    ElementRef,
    EventEmitter,
    Input,
    OnChanges,
    OnInit,
    Output,
    SimpleChanges,
    ViewChild
} from "@angular/core";
import {File} from "../../../../../../api/models/File";
import {FileCardComponent} from "../../file-card/file-card.component";
import {CdkVirtualScrollViewport} from "@angular/cdk/scrolling";
import {TabService} from "../../../../../services/tab/tab.service";
import {FileService} from "../../../../../services/file/file.service";
import {Selectable} from "../../../../../models/Selectable";
import {Key} from "w3c-keys";

@Component({
    selector: "app-file-grid",
    templateUrl: "./file-grid.component.html",
    styleUrls: ["./file-grid.component.scss"]
})
export class FileGridComponent implements OnChanges, OnInit, AfterViewInit {

    @Input() files: File[] = [];
    @Input() columns: number = 6;
    @Input() preselectedFile: File | undefined;
    @Output() fileOpen = new EventEmitter<File>();
    @Output() fileSelect = new EventEmitter<File[]>();
    @Output() fileDelete = new EventEmitter<File[]>();
    @Output() fileDeleted = new EventEmitter<File[]>();

    @ViewChild("virtualScrollGrid") virtualScroll!: CdkVirtualScrollViewport;
    @ViewChild("inner") inner!: ElementRef<HTMLDivElement>;

    selectedEntries: Selectable<File>[] = [];
    partitionedGridEntries: Selectable<File>[][] = [];
    private shiftClicked = false;
    private ctrlClicked = false;
    private gridEntries: Selectable<File>[] = [];

    constructor(
        private tabService: TabService,
        private fileService: FileService,
    ) {
        tabService.selectedTab.subscribe(() => this.adjustElementSizes());
    }

    public ngOnInit(): void {
        this.gridEntries = this.files.map(
            file => new Selectable<File>(file, false));
        this.setPartitionedGridEntries();
    }

    public ngAfterViewInit(): void {
        this.focus();
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (changes["files"]) {
            this.gridEntries = this.files.map(
                file => new Selectable<File>(file, false));
            this.refreshFileSelections();
            this.setPartitionedGridEntries();
        }
    }

    /**
     * File selector logic
     * @param {FileCardComponent} clickedEntry
     */
    setSelectedFile(clickedEntry: Selectable<File>) {
        if (!(this.shiftClicked || this.ctrlClicked) && this.selectedEntries.length > 0) {
            this.selectedEntries.forEach(entry => {
                if (entry !== clickedEntry) entry.selected = false;
            });
            this.selectedEntries = [];
        }
        if (this.shiftClicked && this.selectedEntries.length > 0) {
            this.handleShiftSelect(clickedEntry);
        } else {
            clickedEntry.selected = !clickedEntry.selected;
            if (!clickedEntry.selected) {
                const index = this.selectedEntries.indexOf(clickedEntry);
                if (index > -1) {
                    this.selectedEntries.splice(index, 1);
                }
            } else {
                this.selectedEntries.push(clickedEntry);
            }
        }
        this.fileSelect.emit(this.selectedEntries.map(g => g.data));
    }

    public selectEntryWhenNotSelected(entry: Selectable<File>) {
        if (!entry.selected) {
            this.setSelectedFile(entry);
        }
    }

    public adjustElementSizes(): void {
        if (this.virtualScroll) {
            this.virtualScroll.checkViewportSize();
        }
    }

    public async regenerateThumbnail(files: File[]) {
        for (const file of files) {
            await this.fileService.deleteThumbnails(file);
        }
    }

    public focus() {
        this.inner.nativeElement.focus();
    }

    public handleKeydownEvent(event: KeyboardEvent) {
        this.shiftClicked ||= event.shiftKey;
        this.ctrlClicked ||= event.ctrlKey;

        switch (event.key) {
            case Key.ArrowRight:
                this.handleArrowSelect("right");
                break;
            case Key.ArrowLeft:
                this.handleArrowSelect("left");
                break;
            case Key.ArrowDown:
                this.handleArrowSelect("down");
                break;
            case Key.ArrowUp:
                this.handleArrowSelect("up");
                break;
            case Key.PageDown:
                this.pageDown();
                break;
            case Key.PageUp:
                this.pageUp();
                break;
            case Key.a:
            case Key.A:
                if (this.shiftClicked && this.ctrlClicked) {
                    this.selectNone();
                } else if (this.ctrlClicked) {
                    event.preventDefault();
                    this.selectAll();
                }
                break;
            case Key.Enter:
                if (this.selectedEntries.length === 1) {
                    this.fileOpen.emit(this.selectedEntries[0].data);
                }
                break;
            case Key.Delete:
                this.fileDelete.emit(this.selectedEntries.map(e => e.data));
                break;
        }
    }

    public getSelectedFiles(): File[] {
        return this.selectedEntries.map(e => e.data);
    }

    public handleKeyupEvent(event: KeyboardEvent) {
        this.shiftClicked = event.shiftKey ? false : this.shiftClicked;
        this.ctrlClicked = event.ctrlKey ? false : this.ctrlClicked;
    }

    public trackByFileRowId(index: number, item: Selectable<File>[]) {
        return item.map(e => e.data.id).join("-");
    }

    public trackByFileId(index: number, item: Selectable<File>) {
        return item.data.id;
    }

    private setPartitionedGridEntries() {
        this.partitionedGridEntries = [];
        let scrollToIndex = -1;
        let selectedEntry: Selectable<File> | undefined = undefined;

        for (let i = 0; i < (Math.ceil(
            this.gridEntries.length / this.columns)); i++) {
            const entries = this.gridEntries.slice(
                i * this.columns,
                Math.min(this.gridEntries.length, (i + 1) * this.columns)
            );
            this.partitionedGridEntries.push(entries);

            const preselectedEntry = entries.find(
                e => e.data.id == this.preselectedFile?.id);

            if (preselectedEntry) {
                scrollToIndex = i;
                selectedEntry = preselectedEntry;
            }
        }
        if (scrollToIndex >= 0 && this.preselectedFile && this.selectedEntries.length == 0) {
            setTimeout(() => {  // add timeout to avoid being stuck in the update loop
                if (this.virtualScroll) {
                    this.virtualScroll?.scrollToIndex(scrollToIndex);

                    if (selectedEntry) {
                        selectedEntry.selected = true;
                        this.selectedEntries.push(selectedEntry);
                    }
                }
            }, 0);
        }
    }

    private refreshFileSelections() {
        const newSelection: Selectable<File>[] = this.gridEntries.filter(
            entry => this.selectedEntries.findIndex(
                e => e.data.id == entry.data.id) >= 0);
        newSelection.forEach(entry => entry.selected = true);
        this.selectedEntries = newSelection;
    }

    private handleShiftSelect(clickedEntry: Selectable<File>): void {
        const lastEntry = this.selectedEntries[this.selectedEntries.length - 1];
        let found = false;


        if (clickedEntry == lastEntry) {
            return;
        }

        for (const gridEntry of this.gridEntries) {
            if (found) {
                gridEntry.selected = true;
                this.selectedEntries.push(gridEntry);
                if (gridEntry === clickedEntry || gridEntry == lastEntry) {
                    return;
                }
            } else if (gridEntry === lastEntry || gridEntry === clickedEntry) {
                found = true;
                if (gridEntry === clickedEntry) {
                    gridEntry.selected = true;
                    this.selectedEntries.push(gridEntry);
                }
            }
        }
    }

    private selectAll() {
        this.selectedEntries = this.gridEntries;
        this.gridEntries.forEach(g => g.select());
    }

    private selectNone() {
        this.selectedEntries = [];
        this.gridEntries.forEach(g => g.unselect());
    }

    private handleArrowSelect(direction: "up" | "down" | "left" | "right") {
        if (this.gridEntries.length === 0) {
            return;
        }
        const lastSelectedEntry = this.selectedEntries[this.selectedEntries.length - 1] ?? this.gridEntries[0];
        let selectedIndex = this.gridEntries.indexOf(lastSelectedEntry);

        if (this.selectedEntries.length > 0) {
            switch (direction) {
                case "up":
                    selectedIndex -= this.columns;
                    break;
                case "down":
                    selectedIndex += this.columns;
                    break;
                case "left":
                    selectedIndex--;
                    break;
                case "right":
                    selectedIndex++;
                    break;
            }
            while (selectedIndex < 0) {
                selectedIndex = this.gridEntries.length + selectedIndex;
            }
            if (selectedIndex > this.gridEntries.length) {
                selectedIndex %= this.gridEntries.length;
            }
        }

        this.setSelectedFile(this.gridEntries[selectedIndex]);

        if (this.virtualScroll) {
            const viewportSize = this.virtualScroll.getViewportSize();
            let offsetTop = this.virtualScroll.measureScrollOffset("top");
            const contentOffset = Math.floor(selectedIndex / this.columns) * 260;

            if (contentOffset > offsetTop + viewportSize - 300 || contentOffset < offsetTop) {
                this.virtualScroll.scrollToIndex(Math.floor(selectedIndex / this.columns));

                offsetTop = this.virtualScroll.measureScrollOffset("top");
                if (contentOffset < offsetTop + (viewportSize / 2)) {
                    this.virtualScroll.scrollToOffset((offsetTop + 130) - viewportSize / 2);
                }
            }
        }
    }

    private pageDown() {
        if (this.virtualScroll) {
            const offsetTop = this.virtualScroll.measureScrollOffset("top");
            this.virtualScroll.scrollToOffset(offsetTop + this.virtualScroll.getViewportSize());
        }
    }

    private pageUp() {
        if (this.virtualScroll) {
            const offsetTop = this.virtualScroll.measureScrollOffset("top");
            this.virtualScroll.scrollToOffset(offsetTop - this.virtualScroll.getViewportSize());
        }
    }
}
