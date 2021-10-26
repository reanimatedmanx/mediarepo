import { Injectable } from '@angular/core';
import {RepositoryService} from "../repository/repository.service";
import {BehaviorSubject} from "rxjs";
import {ErrorBrokerService} from "../error-broker/error-broker.service";
import {TagService} from "../tag/tag.service";
import {FileService} from "../file/file.service";

@Injectable({
  providedIn: 'root'
})
export class DataloaderService {

  constructor(
    private erroBroker: ErrorBrokerService,
    private repositoryService: RepositoryService,
    private fileService: FileService,
    private tagService: TagService) { }

  public async loadData() {
    try {
      await this.repositoryService.loadRepositories();
      await this.tagService.loadTags();
      await this.fileService.getFiles();
    } catch (err) {
      this.erroBroker.showError(err);
    }
  }
}
