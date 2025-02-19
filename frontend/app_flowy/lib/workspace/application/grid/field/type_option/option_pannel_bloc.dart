import 'package:flowy_sdk/protobuf/flowy-grid/selection_type_option.pb.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'dart:async';
import 'package:dartz/dartz.dart';
part 'option_pannel_bloc.freezed.dart';

class OptionPannelBloc extends Bloc<OptionPannelEvent, OptionPannelState> {
  OptionPannelBloc({required List<SelectOption> options}) : super(OptionPannelState.initial(options)) {
    on<OptionPannelEvent>(
      (event, emit) async {
        await event.map(
          createOption: (_CreateOption value) async {
            emit(state.copyWith(isEditingOption: false, newOptionName: Some(value.optionName)));
          },
          beginAddingOption: (_BeginAddingOption value) {
            emit(state.copyWith(isEditingOption: true, newOptionName: none()));
          },
          endAddingOption: (_EndAddingOption value) {
            emit(state.copyWith(isEditingOption: false, newOptionName: none()));
          },
          updateOption: (_UpdateOption value) {
            emit(state.copyWith(updateOption: Some(value.option)));
          },
          deleteOption: (_DeleteOption value) {
            emit(state.copyWith(deleteOption: Some(value.option)));
          },
        );
      },
    );
  }

  @override
  Future<void> close() async {
    return super.close();
  }
}

@freezed
class OptionPannelEvent with _$OptionPannelEvent {
  const factory OptionPannelEvent.createOption(String optionName) = _CreateOption;
  const factory OptionPannelEvent.beginAddingOption() = _BeginAddingOption;
  const factory OptionPannelEvent.endAddingOption() = _EndAddingOption;
  const factory OptionPannelEvent.updateOption(SelectOption option) = _UpdateOption;
  const factory OptionPannelEvent.deleteOption(SelectOption option) = _DeleteOption;
}

@freezed
class OptionPannelState with _$OptionPannelState {
  const factory OptionPannelState({
    required List<SelectOption> options,
    required bool isEditingOption,
    required Option<String> newOptionName,
    required Option<SelectOption> updateOption,
    required Option<SelectOption> deleteOption,
  }) = _OptionPannelState;

  factory OptionPannelState.initial(List<SelectOption> options) => OptionPannelState(
        options: options,
        isEditingOption: false,
        newOptionName: none(),
        updateOption: none(),
        deleteOption: none(),
      );
}
