import {NgModule} from "@angular/core";
import {CommonModule} from "@angular/common";
import {AddRepositoryDialogComponent} from "./add-repository-dialog/add-repository-dialog.component";
import {RepositoryFormComponent} from "./repository-form/repository-form.component";
import {EditRepositoryDialogComponent} from "./edit-repository-dialog/edit-repository-dialog.component";
import {MatDialogModule} from "@angular/material/dialog";
import {MatButtonModule} from "@angular/material/button";
import {MatTooltipModule} from "@angular/material/tooltip";
import {MatSelectModule} from "@angular/material/select";
import {MatInputModule} from "@angular/material/input";
import {ReactiveFormsModule} from "@angular/forms";
import {NgIconsModule} from "@ng-icons/core";
import {MatFolder} from "@ng-icons/material-icons/baseline";


@NgModule({
    declarations: [
        AddRepositoryDialogComponent,
        EditRepositoryDialogComponent,
        RepositoryFormComponent
    ],
    exports: [
        AddRepositoryDialogComponent,
        EditRepositoryDialogComponent,
    ],
    imports: [
        CommonModule,
        MatDialogModule,
        MatButtonModule,
        MatTooltipModule,
        MatSelectModule,
        MatInputModule,
        ReactiveFormsModule,
        NgIconsModule.withIcons({ MatFolder })
    ]
})
export class RepositoryModule {
}
