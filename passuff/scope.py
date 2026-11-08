"""Watch scope and focus editing."""

from __future__ import annotations

from pathlib import Path

from .config import ConfigError, RELATIONS, AppConfig, update_focus, update_scope


class ScopeEditor:
    def __init__(self, config: AppConfig, path: Path) -> None:
        self.config = config
        self.path = path

    def select(self, repository: str) -> None:
        repo = repository.strip()
        if repo.count("/") != 1 or any(ch.isspace() for ch in repo):
            raise ConfigError("repository must be owner/repo")
        repositories = list(self.config.watch.repositories)
        if repo.casefold() not in {item.casefold() for item in repositories}:
            repositories.append(repo)
        update_scope(self.path, "selected", repositories)
        self.config.watch.mode = "selected"
        self.config.watch.repositories = repositories

    def deselect(self, repository: str) -> None:
        if self.config.watch.mode == "all":
            raise ConfigError("cannot deselect in all mode")
        repositories = [
            item for item in self.config.watch.repositories if item.casefold() != repository.casefold()
        ]
        update_scope(self.path, "selected", repositories)
        self.config.watch.repositories = repositories

    def set_mode(self, mode: str) -> None:
        if mode not in {"all", "selected"}:
            raise ConfigError("watch mode must be all or selected")
        update_scope(self.path, mode, list(self.config.watch.repositories))
        self.config.watch.mode = mode

    def set_focus(self, relations: list[str]) -> None:
        if not relations or any(rel not in RELATIONS for rel in relations):
            raise ConfigError("invalid focus relation")
        update_focus(self.path, relations)
        self.config.focus.relations = list(relations)
