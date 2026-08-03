use super::{
    App, ConditionFamily, DraftBinding, HistoryEntry, RespondDraftField, RuleMatchDraftField,
    TransientOperation, workspace_session,
};
use apimokka_model::workspace_port::{
    map_body_condition, map_header_condition, map_response, map_root_setting, map_rule_match,
};
use apimokka_model::{
    BodyConditionPayload, CollectionEdit, EditIntent, HeaderConditionPayload, NodeId, ResponseMode,
    RuleEditPayload, RuntimeEffect, WorkspaceEditValue, WorkspaceNodeKind, WorkspaceRootKey,
};

impl App {
    pub(crate) fn update_root_setting(&mut self, key: WorkspaceRootKey, value: WorkspaceEditValue) {
        let operation = TransientOperation::Root(key);
        let before = self.current_root_edit(key);
        let edit = match map_root_setting(key, value) {
            Ok(edit) => edit,
            Err(error) => {
                self.present_operation_problem(
                    operation,
                    "Workspace setting rejected",
                    error.to_string(),
                );
                return;
            }
        };
        if before.as_ref().is_some_and(|before| before == &edit) {
            self.sync_root_setting_draft(key);
            self.clear_operation_problem(operation);
            return;
        }
        let effect = edit.effect();
        let history_after = edit.clone();
        if self
            .apply_workspace_operation(operation, EditIntent::UpdateRootSetting(edit))
            .is_some()
        {
            self.clear_operation_problem(operation);
            self.sync_root_setting_draft(key);
            if let Some(before) = before {
                self.push_undo(HistoryEntry::RootSetting {
                    before,
                    after: history_after,
                });
            }
            match effect {
                RuntimeEffect::None => {}
                RuntimeEffect::Reload => self.trigger_reload(),
                RuntimeEffect::Restart => self.trigger_restart(),
            }
        }
    }

    fn current_root_edit(&self, key: WorkspaceRootKey) -> Option<apimokka_model::RootSettingEdit> {
        let root = &self.snapshot.as_ref()?.root_settings;
        let value = match key {
            WorkspaceRootKey::ListenerIpAddress => {
                WorkspaceEditValue::String(root.listener_ip.clone())
            }
            WorkspaceRootKey::ListenerPort => {
                WorkspaceEditValue::Integer(i64::from(root.listener_port))
            }
            WorkspaceRootKey::ServiceStrategy => {
                WorkspaceEditValue::Enum(root.strategy.label().into())
            }
            WorkspaceRootKey::TlsEnabled => WorkspaceEditValue::Boolean(root.tls_enabled),
            WorkspaceRootKey::LogLevel => WorkspaceEditValue::Enum(root.log_level.clone()),
            _ => return None,
        };
        map_root_setting(key, value).ok()
    }

    pub(crate) fn update_rule_prototype(
        &mut self,
        mutate: impl FnOnce(&mut workspace_session::RulePrototype),
    ) {
        let Some(rule_id) = self.selection.rule else {
            return;
        };
        let Some(session) = self.snapshot.as_mut() else {
            return;
        };
        let prototype = session.prototype.rule_extras.entry(rule_id).or_default();
        let before = prototype.clone();
        mutate(prototype);
        let after = prototype.clone();
        if before != after {
            self.push_undo(HistoryEntry::RulePrototype {
                rule_id,
                before,
                after,
            });
        }
    }

    fn selected_rule_id(&self) -> Option<NodeId> {
        self.selection.rule
    }

    pub(crate) fn current_rule_edit(&self, id: NodeId) -> Option<RuleEditPayload> {
        let rule = self.snapshot.as_ref()?.latest().rule(id)?;
        Some(RuleEditPayload {
            rule_match: rule.rule_match().clone(),
            headers: CollectionEdit::Preserve,
            body: CollectionEdit::Preserve,
            respond: rule.respond().clone(),
        })
    }

    pub(crate) fn update_rule_core(
        &mut self,
        field: RuleMatchDraftField,
        mutate: impl FnOnce(&mut apimokka_model::RulePayload),
    ) {
        let Some(id) = self.selected_rule_id() else {
            return;
        };
        let operation = TransientOperation::RuleMatch { rule_id: id, field };
        let before = self
            .snapshot
            .as_ref()
            .and_then(|session| session.latest().rule(id))
            .map(|rule| rule.rule_match().clone());
        let Some(session) = self.snapshot.as_mut() else {
            return;
        };
        let Some(draft) = session.ensure_rule_draft(id) else {
            return;
        };
        mutate(&mut draft.payload);
        let draft = draft.payload.clone();
        let rule_match = match map_rule_match(&draft.url_path, draft.url_path_op, &draft.method) {
            Ok(value) => value,
            Err(error) => {
                self.present_operation_problem(operation, "Rule edit rejected", error.to_string());
                return;
            }
        };
        let Some(mut rule) = self.current_rule_edit(id) else {
            return;
        };
        if rule.rule_match == rule_match {
            self.sync_rule_match_draft(id, field);
            self.clear_operation_problem(operation);
            return;
        }
        rule.rule_match = rule_match.clone();
        if self
            .apply_workspace_operation(operation, EditIntent::UpdateRule { id, rule })
            .is_some()
        {
            self.clear_operation_problem(operation);
            self.sync_rule_match_draft(id, field);
            if let Some(before) = before {
                self.push_undo(HistoryEntry::RuleMatch {
                    rule_id: id,
                    field,
                    before,
                    after: rule_match,
                });
            }
        }
    }

    pub(crate) fn update_response_draft(
        &mut self,
        field: RespondDraftField,
        mutate: impl FnOnce(&mut apimokka_model::respond::RespondPayload),
    ) {
        let Some(id) = self.selected_rule_id() else {
            return;
        };
        let operation = TransientOperation::Respond { rule_id: id, field };
        let before = self
            .snapshot
            .as_ref()
            .and_then(|session| session.latest().rule(id))
            .map(|rule| rule.respond().clone());
        let Some(session) = self.snapshot.as_mut() else {
            return;
        };
        let Some(draft) = session.ensure_rule_draft(id) else {
            return;
        };
        mutate(&mut draft.payload.respond);
        let response = draft.payload.respond.clone();
        let delay = draft.response_delay.clone();
        let mode = match response.mode {
            apimokka_model::snapshot::RespondMode::InlineText => ResponseMode::Inline,
            apimokka_model::snapshot::RespondMode::ServeFile => ResponseMode::File,
        };
        let mapped = match map_response(
            mode,
            &response.text,
            &response.file_path,
            &response.status,
            &delay,
        ) {
            Ok(value) => value,
            Err(error) => {
                self.present_operation_problem(
                    operation,
                    "Response edit rejected",
                    error.to_string(),
                );
                return;
            }
        };
        if self
            .snapshot
            .as_ref()
            .and_then(|session| session.latest().rule(id))
            .is_some_and(|rule| rule.respond() == &mapped)
        {
            self.sync_respond_draft(id, field);
            self.clear_operation_problem(operation);
            return;
        }
        if self
            .apply_workspace_operation(
                operation,
                EditIntent::UpdateRespond {
                    id,
                    respond: mapped.clone(),
                },
            )
            .is_some()
        {
            self.clear_operation_problem(operation);
            self.sync_respond_draft(id, field);
            if let Some(before) = before {
                self.push_undo(HistoryEntry::Respond {
                    rule_id: id,
                    field,
                    before,
                    after: mapped,
                });
            }
        }
    }

    pub(crate) fn update_response_delay_draft(&mut self, value: String) {
        let Some(id) = self.selected_rule_id() else {
            return;
        };
        let operation = TransientOperation::Respond {
            rule_id: id,
            field: RespondDraftField::Delay,
        };
        let before = self
            .snapshot
            .as_ref()
            .and_then(|session| session.latest().rule(id))
            .map(|rule| rule.respond().clone());
        let Some(session) = self.snapshot.as_mut() else {
            return;
        };
        let Some(draft) = session.ensure_rule_draft(id) else {
            return;
        };
        draft.response_delay = value;
        let response = draft.payload.respond.clone();
        let delay = draft.response_delay.clone();
        let mode = match response.mode {
            apimokka_model::snapshot::RespondMode::InlineText => ResponseMode::Inline,
            apimokka_model::snapshot::RespondMode::ServeFile => ResponseMode::File,
        };
        let mapped = match map_response(
            mode,
            &response.text,
            &response.file_path,
            &response.status,
            &delay,
        ) {
            Ok(value) => value,
            Err(error) => {
                self.present_operation_problem(
                    operation,
                    "Response edit rejected",
                    error.to_string(),
                );
                return;
            }
        };
        if before.as_ref().is_some_and(|before| before == &mapped) {
            self.sync_respond_draft(id, RespondDraftField::Delay);
            self.clear_operation_problem(operation);
            return;
        }
        if self
            .apply_workspace_operation(
                operation,
                EditIntent::UpdateRespond {
                    id,
                    respond: mapped.clone(),
                },
            )
            .is_some()
        {
            self.clear_operation_problem(operation);
            self.sync_respond_draft(id, RespondDraftField::Delay);
            if let Some(before) = before {
                self.push_undo(HistoryEntry::Respond {
                    rule_id: id,
                    field: RespondDraftField::Delay,
                    before,
                    after: mapped,
                });
            }
        }
    }

    pub(crate) fn add_header_draft(&mut self) {
        let Some(id) = self.selected_rule_id() else {
            return;
        };
        let Some(session) = self.snapshot.as_mut() else {
            return;
        };
        let key = session.creation_key("header");
        if let Some(draft) = session.ensure_rule_draft(id) {
            draft.push_header(key.clone());
            session.focus_condition(id, ConditionFamily::Header, DraftBinding::Pending(key));
        }
    }

    pub(crate) fn update_header_draft(
        &mut self,
        index: usize,
        mutate: impl FnOnce(&mut HeaderConditionPayload),
    ) {
        let Some(id) = self.selected_rule_id() else {
            return;
        };
        let operation = TransientOperation::Header { rule_id: id, index };
        let Some(session) = self.snapshot.as_mut() else {
            return;
        };
        let Some(draft) = session.ensure_rule_draft(id) else {
            return;
        };
        let (Some(condition), Some(binding)) = (
            draft.payload.headers.get_mut(index),
            draft.header_bindings.get(index).cloned(),
        ) else {
            return;
        };
        mutate(condition);
        let condition = condition.clone();
        session.focus_condition(id, ConditionFamily::Header, binding.clone());
        let mapped = match map_header_condition(&condition.name, condition.op, &condition.value) {
            Ok(value) => value,
            Err(error) => {
                self.present_operation_problem(
                    operation,
                    "Header condition rejected",
                    error.to_string(),
                );
                return;
            }
        };
        let before = match &binding {
            DraftBinding::Existing(condition_id) => self
                .snapshot
                .as_ref()
                .and_then(|session| session.latest().rule(id))
                .and_then(|rule| {
                    rule.conditions()
                        .headers
                        .iter()
                        .find(|candidate| candidate.id == *condition_id)
                })
                .map(|candidate| candidate.condition.clone()),
            DraftBinding::Pending(_) => None,
        };
        if before.as_ref().is_some_and(|before| before == &mapped) {
            if let DraftBinding::Existing(condition_id) = binding {
                self.sync_header_draft_condition(id, index, condition_id);
            }
            self.clear_operation_problem(operation);
            return;
        }
        let intent = match &binding {
            DraftBinding::Existing(id) => EditIntent::UpdateHeaderCondition {
                id: *id,
                condition: mapped.clone(),
            },
            DraftBinding::Pending(key) => EditIntent::AddHeaderCondition {
                rule_id: id,
                condition: mapped.clone(),
                key: key.clone(),
            },
        };
        let Some(outcome) = self.apply_workspace_operation(operation, intent) else {
            return;
        };
        self.clear_operation_problem(operation);
        if matches!(binding, DraftBinding::Pending(_)) {
            if let Some(receipt) = outcome
                .creations
                .iter()
                .find(|receipt| receipt.kind == WorkspaceNodeKind::HeaderCondition)
            {
                if let Some(draft) = self
                    .snapshot
                    .as_mut()
                    .and_then(|session| session.rule_drafts.get_mut(&id))
                {
                    draft.header_bindings[index] = DraftBinding::Existing(receipt.new_id);
                }
                self.sync_header_draft_condition(id, index, receipt.new_id);
                if let DraftBinding::Pending(key) = binding {
                    self.push_undo(HistoryEntry::HeaderAdd {
                        rule_id: id,
                        key,
                        condition: mapped,
                        current_id: receipt.new_id,
                    });
                }
            }
        } else if let (DraftBinding::Existing(current_id), Some(before)) = (binding, before) {
            self.sync_header_draft_condition(id, index, current_id);
            self.push_undo(HistoryEntry::HeaderUpdate {
                current_id,
                before,
                after: mapped,
            });
        }
    }

    pub(crate) fn remove_header_draft(&mut self, index: usize) {
        let Some(id) = self.selected_rule_id() else {
            return;
        };
        let binding = self
            .snapshot
            .as_mut()
            .and_then(|session| session.ensure_rule_draft(id))
            .and_then(|draft| draft.header_bindings.get(index).cloned());
        let Some(binding) = binding else { return };
        let canonical = match binding {
            DraftBinding::Existing(condition_id) => self
                .snapshot
                .as_ref()
                .and_then(|session| session.latest().rule(id))
                .and_then(|rule| {
                    rule.conditions()
                        .headers
                        .iter()
                        .find(|candidate| candidate.id == condition_id)
                })
                .map(|candidate| (condition_id, candidate.condition.clone())),
            DraftBinding::Pending(_) => None,
        };
        let removed = match &binding {
            DraftBinding::Existing(condition_id) => self
                .apply_workspace_intent(EditIntent::RemoveHeaderCondition { id: *condition_id })
                .is_some(),
            DraftBinding::Pending(_) => true,
        };
        if removed {
            if let Some(session) = self.snapshot.as_mut() {
                session.clear_condition_focus_family(id, ConditionFamily::Header);
            }
            if let Some(draft) = self
                .snapshot
                .as_mut()
                .and_then(|session| session.rule_drafts.get_mut(&id))
            {
                draft.payload.headers.remove(index);
                draft.header_bindings.remove(index);
            }
            if let Some((current_id, condition)) = canonical {
                let key = self
                    .snapshot
                    .as_mut()
                    .unwrap()
                    .creation_key("history-header");
                self.push_undo(HistoryEntry::HeaderRemove {
                    rule_id: id,
                    index,
                    key,
                    condition,
                    current_id,
                });
            }
        }
    }

    fn sync_header_draft_condition(
        &mut self,
        rule_id: NodeId,
        draft_index: usize,
        condition_id: NodeId,
    ) {
        let projected = self.snapshot.as_ref().and_then(|session| {
            let canonical = session.latest().rule(rule_id)?;
            let position = canonical
                .conditions()
                .headers
                .iter()
                .position(|condition| condition.id == condition_id)?;
            session
                .find_rule(rule_id)?
                .1
                .payload
                .headers
                .get(position)
                .cloned()
        });
        if let (Some(projected), Some(draft)) = (
            projected,
            self.snapshot
                .as_mut()
                .and_then(|session| session.rule_drafts.get_mut(&rule_id)),
        ) && let Some(condition) = draft.payload.headers.get_mut(draft_index)
        {
            *condition = projected;
        }
    }

    pub(crate) fn clear_header_drafts(&mut self) {
        let Some(id) = self.selected_rule_id() else {
            return;
        };
        if let Some(session) = self.snapshot.as_mut() {
            session.clear_condition_focus_family(id, ConditionFamily::Header);
        }
        let bindings = self
            .snapshot
            .as_mut()
            .and_then(|session| session.ensure_rule_draft(id))
            .map(|draft| draft.header_bindings.clone())
            .unwrap_or_default();
        let removed = self
            .snapshot
            .as_ref()
            .and_then(|session| session.latest().rule(id))
            .map(|rule| {
                rule.conditions()
                    .headers
                    .iter()
                    .enumerate()
                    .map(|(index, condition)| (index, condition.id, condition.condition.clone()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let intents = bindings
            .iter()
            .filter_map(|binding| match binding {
                DraftBinding::Existing(id) => Some(EditIntent::RemoveHeaderCondition { id: *id }),
                DraftBinding::Pending(_) => None,
            })
            .collect::<Vec<_>>();
        if !intents.is_empty() && self.apply_workspace_transaction(intents).is_none() {
            return;
        }
        if let Some(draft) = self
            .snapshot
            .as_mut()
            .and_then(|session| session.rule_drafts.get_mut(&id))
        {
            draft.payload.headers.clear();
            draft.header_bindings.clear();
        }
        if !removed.is_empty() {
            let entries = removed
                .into_iter()
                .map(|(index, current_id, condition)| {
                    let key = self
                        .snapshot
                        .as_mut()
                        .unwrap()
                        .creation_key("history-header");
                    (index, key, condition, current_id)
                })
                .collect();
            self.push_undo(HistoryEntry::HeadersClear {
                rule_id: id,
                entries,
            });
        }
    }

    pub(crate) fn add_body_draft(&mut self) {
        let Some(id) = self.selected_rule_id() else {
            return;
        };
        let Some(session) = self.snapshot.as_mut() else {
            return;
        };
        let key = session.creation_key("body");
        if let Some(draft) = session.ensure_rule_draft(id) {
            draft.push_body(key.clone());
            session.focus_condition(id, ConditionFamily::Body, DraftBinding::Pending(key));
        }
    }

    pub(crate) fn update_body_draft(
        &mut self,
        index: usize,
        mutate: impl FnOnce(&mut BodyConditionPayload),
    ) {
        let Some(id) = self.selected_rule_id() else {
            return;
        };
        let operation = TransientOperation::Body { rule_id: id, index };
        let Some(session) = self.snapshot.as_mut() else {
            return;
        };
        let Some(draft) = session.ensure_rule_draft(id) else {
            return;
        };
        let (Some(condition), Some(binding)) = (
            draft.payload.body.get_mut(index),
            draft.body_bindings.get(index).cloned(),
        ) else {
            return;
        };
        mutate(condition);
        let condition = condition.clone();
        session.focus_condition(id, ConditionFamily::Body, binding.clone());
        let mapped = match map_body_condition(&condition.path, condition.op, &condition.value) {
            Ok(value) => value,
            Err(error) => {
                self.present_operation_problem(
                    operation,
                    "Body condition rejected",
                    error.to_string(),
                );
                return;
            }
        };
        let before = match &binding {
            DraftBinding::Existing(condition_id) => self
                .snapshot
                .as_ref()
                .and_then(|session| session.latest().rule(id))
                .and_then(|rule| {
                    rule.conditions()
                        .body
                        .iter()
                        .find(|candidate| candidate.id == *condition_id)
                })
                .map(|candidate| candidate.condition.clone()),
            DraftBinding::Pending(_) => None,
        };
        if before.as_ref().is_some_and(|before| before == &mapped) {
            if let DraftBinding::Existing(condition_id) = binding {
                self.sync_body_draft_condition(id, index, condition_id);
            }
            self.clear_operation_problem(operation);
            return;
        }
        let intent = match &binding {
            DraftBinding::Existing(id) => EditIntent::UpdateBodyCondition {
                id: *id,
                condition: mapped.clone(),
            },
            DraftBinding::Pending(key) => EditIntent::AddBodyCondition {
                rule_id: id,
                condition: mapped.clone(),
                key: key.clone(),
            },
        };
        let Some(outcome) = self.apply_workspace_operation(operation, intent) else {
            return;
        };
        self.clear_operation_problem(operation);
        if matches!(binding, DraftBinding::Pending(_)) {
            if let Some(receipt) = outcome
                .creations
                .iter()
                .find(|receipt| receipt.kind == WorkspaceNodeKind::BodyCondition)
            {
                if let Some(draft) = self
                    .snapshot
                    .as_mut()
                    .and_then(|session| session.rule_drafts.get_mut(&id))
                {
                    draft.body_bindings[index] = DraftBinding::Existing(receipt.new_id);
                }
                self.sync_body_draft_condition(id, index, receipt.new_id);
                if let DraftBinding::Pending(key) = binding {
                    self.push_undo(HistoryEntry::BodyAdd {
                        rule_id: id,
                        key,
                        condition: mapped,
                        current_id: receipt.new_id,
                    });
                }
            }
        } else if let (DraftBinding::Existing(current_id), Some(before)) = (binding, before) {
            self.sync_body_draft_condition(id, index, current_id);
            self.push_undo(HistoryEntry::BodyUpdate {
                current_id,
                before,
                after: mapped,
            });
        }
    }

    pub(crate) fn remove_body_draft(&mut self, index: usize) {
        let Some(id) = self.selected_rule_id() else {
            return;
        };
        let binding = self
            .snapshot
            .as_mut()
            .and_then(|session| session.ensure_rule_draft(id))
            .and_then(|draft| draft.body_bindings.get(index).cloned());
        let Some(binding) = binding else { return };
        let canonical = match binding {
            DraftBinding::Existing(condition_id) => self
                .snapshot
                .as_ref()
                .and_then(|session| session.latest().rule(id))
                .and_then(|rule| {
                    rule.conditions()
                        .body
                        .iter()
                        .find(|candidate| candidate.id == condition_id)
                })
                .map(|candidate| (condition_id, candidate.condition.clone())),
            DraftBinding::Pending(_) => None,
        };
        let removed = match &binding {
            DraftBinding::Existing(condition_id) => self
                .apply_workspace_intent(EditIntent::RemoveBodyCondition { id: *condition_id })
                .is_some(),
            DraftBinding::Pending(_) => true,
        };
        if removed {
            if let Some(session) = self.snapshot.as_mut() {
                session.clear_condition_focus_family(id, ConditionFamily::Body);
            }
            if let Some(draft) = self
                .snapshot
                .as_mut()
                .and_then(|session| session.rule_drafts.get_mut(&id))
            {
                draft.payload.body.remove(index);
                draft.body_bindings.remove(index);
            }
            if let Some((current_id, condition)) = canonical {
                let key = self.snapshot.as_mut().unwrap().creation_key("history-body");
                self.push_undo(HistoryEntry::BodyRemove {
                    rule_id: id,
                    index,
                    key,
                    condition,
                    current_id,
                });
            }
        }
    }

    fn sync_body_draft_condition(
        &mut self,
        rule_id: NodeId,
        draft_index: usize,
        condition_id: NodeId,
    ) {
        let projected = self.snapshot.as_ref().and_then(|session| {
            let canonical = session.latest().rule(rule_id)?;
            let position = canonical
                .conditions()
                .body
                .iter()
                .position(|condition| condition.id == condition_id)?;
            session
                .find_rule(rule_id)?
                .1
                .payload
                .body
                .get(position)
                .cloned()
        });
        if let (Some(projected), Some(draft)) = (
            projected,
            self.snapshot
                .as_mut()
                .and_then(|session| session.rule_drafts.get_mut(&rule_id)),
        ) && let Some(condition) = draft.payload.body.get_mut(draft_index)
        {
            *condition = projected;
        }
    }

    pub(crate) fn clear_body_drafts(&mut self) {
        let Some(id) = self.selected_rule_id() else {
            return;
        };
        if let Some(session) = self.snapshot.as_mut() {
            session.clear_condition_focus_family(id, ConditionFamily::Body);
        }
        let bindings = self
            .snapshot
            .as_mut()
            .and_then(|session| session.ensure_rule_draft(id))
            .map(|draft| draft.body_bindings.clone())
            .unwrap_or_default();
        let removed = self
            .snapshot
            .as_ref()
            .and_then(|session| session.latest().rule(id))
            .map(|rule| {
                rule.conditions()
                    .body
                    .iter()
                    .enumerate()
                    .map(|(index, condition)| (index, condition.id, condition.condition.clone()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let intents = bindings
            .iter()
            .filter_map(|binding| match binding {
                DraftBinding::Existing(id) => Some(EditIntent::RemoveBodyCondition { id: *id }),
                DraftBinding::Pending(_) => None,
            })
            .collect::<Vec<_>>();
        if !intents.is_empty() && self.apply_workspace_transaction(intents).is_none() {
            return;
        }
        if let Some(draft) = self
            .snapshot
            .as_mut()
            .and_then(|session| session.rule_drafts.get_mut(&id))
        {
            draft.payload.body.clear();
            draft.body_bindings.clear();
        }
        if !removed.is_empty() {
            let entries = removed
                .into_iter()
                .map(|(index, current_id, condition)| {
                    let key = self.snapshot.as_mut().unwrap().creation_key("history-body");
                    (index, key, condition, current_id)
                })
                .collect();
            self.push_undo(HistoryEntry::BodiesClear {
                rule_id: id,
                entries,
            });
        }
    }
}
