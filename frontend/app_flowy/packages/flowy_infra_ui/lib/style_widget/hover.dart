import 'package:flutter/material.dart';
// ignore: unused_import
import 'package:flowy_infra/time/duration.dart';

typedef HoverBuilder = Widget Function(BuildContext context, bool onHover);

class FlowyHover extends StatefulWidget {
  final HoverDisplayConfig config;
  final HoverBuilder builder;
  final bool Function()? setSelected;

  const FlowyHover({
    Key? key,
    required this.builder,
    required this.config,
    this.setSelected,
  }) : super(key: key);

  @override
  State<FlowyHover> createState() => _FlowyHoverState();
}

class _FlowyHoverState extends State<FlowyHover> {
  bool _onHover = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (p) => setState(() => _onHover = true),
      onExit: (p) => setState(() => _onHover = false),
      child: render(),
    );
  }

  Widget render() {
    var showHover = _onHover;
    if (!showHover && widget.setSelected != null) {
      showHover = widget.setSelected!();
    }

    if (showHover) {
      return FlowyHoverContainer(
        config: widget.config,
        child: widget.builder(context, _onHover),
      );
    } else {
      return widget.builder(context, _onHover);
    }
  }
}

class HoverDisplayConfig {
  final Color borderColor;
  final double borderWidth;
  final Color hoverColor;
  final BorderRadius borderRadius;

  const HoverDisplayConfig(
      {this.borderColor = Colors.transparent,
      this.borderWidth = 0,
      this.borderRadius = const BorderRadius.all(Radius.circular(6)),
      required this.hoverColor});
}

class FlowyHoverContainer extends StatelessWidget {
  final HoverDisplayConfig config;
  final Widget child;

  const FlowyHoverContainer({
    Key? key,
    required this.child,
    required this.config,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final hoverBorder = Border.all(
      color: config.borderColor,
      width: config.borderWidth,
    );

    return Container(
      decoration: BoxDecoration(
        border: hoverBorder,
        color: config.hoverColor,
        borderRadius: config.borderRadius,
      ),
      child: child,
    );
  }
}
